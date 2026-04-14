use async_graphql::{Context, InputObject, Object, ID};
use chrono::Utc;
use uuid::Uuid;

use crate::graphql::query::*;
use crate::middleware::SubjectContext;
use crate::platform_events::{EventResource, PlatformEvent};
use crate::AppState;

// ── Input Types ─────────────────────────────────────────────────────

#[derive(InputObject)]
pub struct CreateDocumentInput {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub content: String,
    pub language: Option<String>,
    pub category_id: Option<ID>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateDocumentInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub language: Option<String>,
    pub category_id: Option<ID>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    pub change_summary: Option<String>,
}

#[derive(InputObject)]
pub struct CreateDocumentCategoryInput {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<ID>,
}

#[derive(InputObject)]
pub struct CreateDocumentRelationshipInput {
    pub source_document_id: ID,
    pub target_document_id: ID,
    pub relationship_type: String,
}

#[derive(InputObject)]
pub struct CreateGlossaryEntryInput {
    pub term: String,
    pub slug: String,
    pub definition: String,
    pub extended_description: Option<String>,
    pub language: Option<String>,
    pub category_id: Option<ID>,
    pub status: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateGlossaryEntryInput {
    pub term: Option<String>,
    pub definition: Option<String>,
    pub extended_description: Option<String>,
    pub language: Option<String>,
    pub category_id: Option<ID>,
    pub status: Option<String>,
}

#[derive(InputObject)]
pub struct CreateGlossaryCategoryInput {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
}

#[derive(InputObject)]
pub struct UpdateGlossaryCategoryInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(InputObject)]
pub struct CreateGlossaryRelationshipInput {
    pub source_entry_id: ID,
    pub target_entry_id: ID,
    pub relationship_type: String,
}

// ── Helpers ─────────────────────────────────────────────────────────

fn get_subject(ctx: &Context<'_>) -> async_graphql::Result<SubjectContext> {
    ctx.data::<SubjectContext>()
        .cloned()
        .map_err(|_| async_graphql::Error::new("Authentication required"))
}

fn parse_uuid(id: &ID) -> async_graphql::Result<Uuid> {
    Uuid::parse_str(id).map_err(|_| async_graphql::Error::new("Invalid UUID"))
}

// ── MutationRoot ────────────────────────────────────────────────────

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_document(
        &self,
        ctx: &Context<'_>,
        input: CreateDocumentInput,
    ) -> async_graphql::Result<GqlDocument> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let subject_uuid: Uuid = subject.subject_id.parse()?;

        state
            .keto
            .require_permission("document", "*", "write", &subject.subject_id)
            .await?;

        let status = input.status.as_deref().unwrap_or("draft");
        let language = input.language.as_deref().unwrap_or("en");
        let tags = input.tags.unwrap_or_default();
        let category_id = input
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let doc = state
            .repos
            .documents()
            .create(
                &input.title,
                &input.slug,
                input.description.as_deref(),
                &input.content,
                language,
                category_id,
                &tags,
                status,
                subject_uuid,
            )
            .await?;

        // Create initial revision
        let _ = state
            .repos
            .document_revisions()
            .create(doc.id, 1, &doc.title, &doc.content, Some("Initial version"), subject_uuid)
            .await;

        // Fire-and-forget audit + event
        let repos = state.repos.clone();
        let events = state.events.clone();
        let doc_id = doc.id;
        let actor = subject.clone();
        let title = doc.title.clone();
        tokio::spawn(async move {
            if let Err(e) = repos
                .audit()
                .create_audit_log(
                    "document",
                    doc_id,
                    "created",
                    Uuid::parse_str(&actor.subject_id).unwrap_or_default(),
                    actor.ip_address.as_deref(),
                    actor.user_agent.as_deref(),
                    Some(serde_json::json!({"title": title})),
                )
                .await
            {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events
                .publish(&PlatformEvent::new(
                    "adeptus.document.created",
                    &actor.subject_id,
                    EventResource {
                        resource_type: "document".to_string(),
                        resource_id: doc_id.to_string(),
                        resource_name: Some(title),
                        resource_url: None,
                    },
                    serde_json::json!({}),
                    None,
                ))
                .await;
        });

        Ok(doc.into())
    }

    async fn update_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateDocumentInput,
    ) -> async_graphql::Result<GqlDocument> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", &id, "write", &subject.subject_id)
            .await?;

        let category_id = input
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let doc = state
            .repos
            .documents()
            .update(
                doc_id,
                input.title.as_deref(),
                input.description.as_deref(),
                input.content.as_deref(),
                input.language.as_deref(),
                category_id,
                input.tags.as_deref(),
                input.status.as_deref(),
            )
            .await?;

        // Create new revision if content or title changed
        if input.content.is_some() || input.title.is_some() {
            let rev_num = state
                .repos
                .document_revisions()
                .get_latest_revision_number(doc_id)
                .await
                .unwrap_or(0)
                + 1;
            let subject_uuid: Uuid = subject.subject_id.parse().unwrap_or_default();
            let _ = state
                .repos
                .document_revisions()
                .create(
                    doc_id,
                    rev_num,
                    &doc.title,
                    &doc.content,
                    input.change_summary.as_deref(),
                    subject_uuid,
                )
                .await;
        }

        // Fire-and-forget audit + event
        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        let title = doc.title.clone();
        tokio::spawn(async move {
            if let Err(e) = repos
                .audit()
                .create_audit_log(
                    "document",
                    doc_id,
                    "updated",
                    Uuid::parse_str(&actor.subject_id).unwrap_or_default(),
                    actor.ip_address.as_deref(),
                    actor.user_agent.as_deref(),
                    None,
                )
                .await
            {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events
                .publish(&PlatformEvent::new(
                    "adeptus.document.updated",
                    &actor.subject_id,
                    EventResource {
                        resource_type: "document".to_string(),
                        resource_id: doc_id.to_string(),
                        resource_name: Some(title),
                        resource_url: None,
                    },
                    serde_json::json!({}),
                    None,
                ))
                .await;
        });

        Ok(doc.into())
    }

    async fn delete_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", &id, "write", &subject.subject_id)
            .await?;

        state.repos.documents().delete(doc_id).await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            if let Err(e) = repos
                .audit()
                .create_audit_log(
                    "document",
                    doc_id,
                    "deleted",
                    Uuid::parse_str(&actor.subject_id).unwrap_or_default(),
                    actor.ip_address.as_deref(),
                    actor.user_agent.as_deref(),
                    None,
                )
                .await
            {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events
                .publish(&PlatformEvent::new(
                    "adeptus.document.deleted",
                    &actor.subject_id,
                    EventResource {
                        resource_type: "document".to_string(),
                        resource_id: doc_id.to_string(),
                        resource_name: None,
                        resource_url: None,
                    },
                    serde_json::json!({}),
                    None,
                ))
                .await;
        });

        Ok(true)
    }

    async fn publish_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<GqlDocument> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", &id, "publish", &subject.subject_id)
            .await?;

        let doc = state
            .repos
            .documents()
            .set_status(doc_id, "published", Some(Utc::now()))
            .await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        let title = doc.title.clone();
        tokio::spawn(async move {
            if let Err(e) = repos
                .audit()
                .create_audit_log(
                    "document",
                    doc_id,
                    "published",
                    Uuid::parse_str(&actor.subject_id).unwrap_or_default(),
                    actor.ip_address.as_deref(),
                    actor.user_agent.as_deref(),
                    None,
                )
                .await
            {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events
                .publish(&PlatformEvent::new(
                    "adeptus.document.published",
                    &actor.subject_id,
                    EventResource {
                        resource_type: "document".to_string(),
                        resource_id: doc_id.to_string(),
                        resource_name: Some(title),
                        resource_url: None,
                    },
                    serde_json::json!({}),
                    None,
                ))
                .await;
        });

        Ok(doc.into())
    }

    async fn unpublish_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<GqlDocument> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", &id, "publish", &subject.subject_id)
            .await?;

        let doc = state
            .repos
            .documents()
            .set_status(doc_id, "unpublished", None)
            .await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("document", doc_id, "unpublished", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), None).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.document.unpublished", &actor.subject_id, EventResource { resource_type: "document".to_string(), resource_id: doc_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(doc.into())
    }

    async fn archive_document(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<GqlDocument> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", &id, "publish", &subject.subject_id)
            .await?;

        let doc = state
            .repos
            .documents()
            .set_status(doc_id, "archived", None)
            .await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("document", doc_id, "archived", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), None).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.document.archived", &actor.subject_id, EventResource { resource_type: "document".to_string(), resource_id: doc_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(doc.into())
    }

    async fn create_document_category(
        &self,
        ctx: &Context<'_>,
        input: CreateDocumentCategoryInput,
    ) -> async_graphql::Result<GqlDocumentCategory> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;

        state
            .keto
            .require_permission("document", "*", "manage", &subject.subject_id)
            .await?;

        let parent_id = input
            .parent_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let cat = state
            .repos
            .document_categories()
            .create(&input.name, &input.slug, input.description.as_deref(), parent_id)
            .await?;

        Ok(cat.into())
    }

    async fn create_document_relationship(
        &self,
        ctx: &Context<'_>,
        input: CreateDocumentRelationshipInput,
    ) -> async_graphql::Result<GqlDocumentRelationship> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let source_id = parse_uuid(&input.source_document_id)?;
        let target_id = parse_uuid(&input.target_document_id)?;

        state
            .keto
            .require_permission(
                "document",
                &input.source_document_id,
                "write",
                &subject.subject_id,
            )
            .await?;

        let rel = state
            .repos
            .document_relationships()
            .create(source_id, target_id, &input.relationship_type)
            .await?;

        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.document.relationship_created", &actor.subject_id, EventResource { resource_type: "document_relationship".to_string(), resource_id: rel.id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({"source": source_id.to_string(), "target": target_id.to_string()}), None)).await;
        });

        Ok(rel.into())
    }

    async fn delete_document_relationship(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let rel_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("document", "*", "write", &subject.subject_id)
            .await?;

        state.repos.document_relationships().delete(rel_id).await?;
        Ok(true)
    }

    async fn delete_file(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let file_id = parse_uuid(&id)?;

        let file = state
            .repos
            .document_files()
            .get_by_id(file_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("File not found"))?;

        state
            .keto
            .require_permission(
                "document",
                &file.document_id.to_string(),
                "write",
                &subject.subject_id,
            )
            .await?;

        state.repos.document_files().delete(file_id).await?;

        // Try to delete from filesystem
        let _ = tokio::fs::remove_file(&file.file_path).await;

        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.file.deleted", &actor.subject_id, EventResource { resource_type: "document_file".to_string(), resource_id: file_id.to_string(), resource_name: Some(file.original_filename), resource_url: None }, serde_json::json!({"document_id": file.document_id.to_string()}), None)).await;
        });

        Ok(true)
    }

    async fn create_glossary_entry(
        &self,
        ctx: &Context<'_>,
        input: CreateGlossaryEntryInput,
    ) -> async_graphql::Result<GqlGlossaryEntry> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let subject_uuid: Uuid = subject.subject_id.parse()?;

        state
            .keto
            .require_permission("glossary", "*", "write", &subject.subject_id)
            .await?;

        let language = input.language.as_deref().unwrap_or("en");
        let status = input.status.as_deref().unwrap_or("draft");
        let category_id = input
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let entry = state
            .repos
            .glossary_entries()
            .create(
                &input.term,
                &input.slug,
                &input.definition,
                input.extended_description.as_deref(),
                language,
                category_id,
                status,
                subject_uuid,
            )
            .await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let entry_id = entry.id;
        let actor = subject.clone();
        let term = entry.term.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("glossary_entry", entry_id, "created", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), Some(serde_json::json!({"term": term}))).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.glossary.created", &actor.subject_id, EventResource { resource_type: "glossary_entry".to_string(), resource_id: entry_id.to_string(), resource_name: Some(term), resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(entry.into())
    }

    async fn update_glossary_entry(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateGlossaryEntryInput,
    ) -> async_graphql::Result<GqlGlossaryEntry> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let entry_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("glossary", &id, "write", &subject.subject_id)
            .await?;

        let category_id = input
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let entry = state
            .repos
            .glossary_entries()
            .update(
                entry_id,
                input.term.as_deref(),
                input.definition.as_deref(),
                input.extended_description.as_deref(),
                input.language.as_deref(),
                category_id,
                input.status.as_deref(),
            )
            .await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        let term = entry.term.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("glossary_entry", entry_id, "updated", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), None).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.glossary.updated", &actor.subject_id, EventResource { resource_type: "glossary_entry".to_string(), resource_id: entry_id.to_string(), resource_name: Some(term), resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(entry.into())
    }

    async fn delete_glossary_entry(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let entry_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("glossary", &id, "write", &subject.subject_id)
            .await?;

        state.repos.glossary_entries().delete(entry_id).await?;

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("glossary_entry", entry_id, "deleted", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), None).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.glossary.deleted", &actor.subject_id, EventResource { resource_type: "glossary_entry".to_string(), resource_id: entry_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(true)
    }

    async fn create_glossary_category(
        &self,
        ctx: &Context<'_>,
        input: CreateGlossaryCategoryInput,
    ) -> async_graphql::Result<GqlGlossaryCategory> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;

        state
            .keto
            .require_permission("glossary", "*", "manage", &subject.subject_id)
            .await?;

        let cat = state
            .repos
            .glossary_categories()
            .create(&input.name, &input.slug, input.description.as_deref())
            .await?;

        let events = state.events.clone();
        let actor = subject.clone();
        let cat_id = cat.id;
        let name = cat.name.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.glossary_category.created", &actor.subject_id, EventResource { resource_type: "glossary_category".to_string(), resource_id: cat_id.to_string(), resource_name: Some(name), resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(cat.into())
    }

    async fn update_glossary_category(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateGlossaryCategoryInput,
    ) -> async_graphql::Result<GqlGlossaryCategory> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let cat_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("glossary", "*", "manage", &subject.subject_id)
            .await?;

        let cat = state
            .repos
            .glossary_categories()
            .update(
                cat_id,
                input.name.as_deref(),
                input.description.as_deref(),
                input.sort_order,
            )
            .await?;

        let events = state.events.clone();
        let actor = subject.clone();
        let name = cat.name.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.glossary_category.updated", &actor.subject_id, EventResource { resource_type: "glossary_category".to_string(), resource_id: cat_id.to_string(), resource_name: Some(name), resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(cat.into())
    }

    async fn delete_glossary_category(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let cat_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("glossary", "*", "manage", &subject.subject_id)
            .await?;

        state.repos.glossary_categories().delete(cat_id).await?;

        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.glossary_category.deleted", &actor.subject_id, EventResource { resource_type: "glossary_category".to_string(), resource_id: cat_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(true)
    }

    async fn create_glossary_relationship(
        &self,
        ctx: &Context<'_>,
        input: CreateGlossaryRelationshipInput,
    ) -> async_graphql::Result<GqlGlossaryRelationship> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let source_id = parse_uuid(&input.source_entry_id)?;
        let target_id = parse_uuid(&input.target_entry_id)?;

        state
            .keto
            .require_permission("glossary", "*", "write", &subject.subject_id)
            .await?;

        let rel = state
            .repos
            .glossary_relationships()
            .create(source_id, target_id, &input.relationship_type)
            .await?;

        let events = state.events.clone();
        let actor = subject.clone();
        let rel_id = rel.id;
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.glossary.relationship_created", &actor.subject_id, EventResource { resource_type: "glossary_relationship".to_string(), resource_id: rel_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({"source": source_id.to_string(), "target": target_id.to_string()}), None)).await;
        });

        Ok(rel.into())
    }

    async fn delete_glossary_relationship(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<bool> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let rel_id = parse_uuid(&id)?;

        state
            .keto
            .require_permission("glossary", "*", "manage", &subject.subject_id)
            .await?;

        state.repos.glossary_relationships().delete(rel_id).await?;

        let events = state.events.clone();
        let actor = subject.clone();
        tokio::spawn(async move {
            events.publish(&PlatformEvent::new("adeptus.glossary.relationship_deleted", &actor.subject_id, EventResource { resource_type: "glossary_relationship".to_string(), resource_id: rel_id.to_string(), resource_name: None, resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(true)
    }

    async fn generate_pdf(
        &self,
        ctx: &Context<'_>,
        document_id: ID,
    ) -> async_graphql::Result<String> {
        let state = ctx.data::<AppState>()?;
        let subject = get_subject(ctx)?;
        let doc_id = parse_uuid(&document_id)?;

        state
            .keto
            .require_permission("document", &document_id, "read", &subject.subject_id)
            .await?;

        let doc = state
            .repos
            .documents()
            .get_by_id(doc_id)
            .await?
            .ok_or_else(|| async_graphql::Error::new("Document not found"))?;

        let latest_rev = state
            .repos
            .document_revisions()
            .get_latest_revision_number(doc_id)
            .await
            .unwrap_or(1);

        let pdf_bytes = crate::handlers::pdf::generate_pdf_bytes(
            &doc.content,
            doc_id,
            &doc.title,
            Some(latest_rev),
            &state.config.pdf,
        )
        .await?;

        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&pdf_bytes);

        let repos = state.repos.clone();
        let events = state.events.clone();
        let actor = subject.clone();
        let title = doc.title.clone();
        tokio::spawn(async move {
            if let Err(e) = repos.audit().create_audit_log("document", doc_id, "pdf_generated", Uuid::parse_str(&actor.subject_id).unwrap_or_default(), actor.ip_address.as_deref(), actor.user_agent.as_deref(), None).await {
                tracing::warn!("Failed to create audit log: {e}");
            }
            events.publish(&PlatformEvent::new("adeptus.document.pdf_generated", &actor.subject_id, EventResource { resource_type: "document".to_string(), resource_id: doc_id.to_string(), resource_name: Some(title), resource_url: None }, serde_json::json!({}), None)).await;
        });

        Ok(encoded)
    }
}
