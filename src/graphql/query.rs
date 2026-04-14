use async_graphql::{Context, ID, InputObject, Object, SimpleObject};
use uuid::Uuid;

use crate::AppState;

// ── GQL Output Types ────────────────────────────────────────────────

#[derive(SimpleObject)]
pub struct GqlDocument {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub content: String,
    pub language: String,
    pub category_id: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub published_at: Option<String>,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::models::Document> for GqlDocument {
    fn from(d: crate::models::Document) -> Self {
        Self {
            id: d.id.to_string(),
            title: d.title,
            slug: d.slug,
            description: d.description,
            content: d.content,
            language: d.language,
            category_id: d.category_id.map(|id| id.to_string()),
            tags: d.tags,
            status: d.status,
            published_at: d.published_at.map(|t| t.to_rfc3339()),
            created_by: d.created_by.to_string(),
            created_at: d.created_at.to_rfc3339(),
            updated_at: d.updated_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlDocumentCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::models::DocumentCategory> for GqlDocumentCategory {
    fn from(c: crate::models::DocumentCategory) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name,
            slug: c.slug,
            description: c.description,
            parent_id: c.parent_id.map(|id| id.to_string()),
            sort_order: c.sort_order,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlDocumentRevision {
    pub id: String,
    pub document_id: String,
    pub revision_number: i32,
    pub title: String,
    pub content: String,
    pub change_summary: Option<String>,
    pub created_by: String,
    pub created_at: String,
}

impl From<crate::models::DocumentRevision> for GqlDocumentRevision {
    fn from(r: crate::models::DocumentRevision) -> Self {
        Self {
            id: r.id.to_string(),
            document_id: r.document_id.to_string(),
            revision_number: r.revision_number,
            title: r.title,
            content: r.content,
            change_summary: r.change_summary,
            created_by: r.created_by.to_string(),
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlDocumentRelationship {
    pub id: String,
    pub source_document_id: String,
    pub target_document_id: String,
    pub relationship_type: String,
    pub created_at: String,
}

impl From<crate::models::DocumentRelationship> for GqlDocumentRelationship {
    fn from(r: crate::models::DocumentRelationship) -> Self {
        Self {
            id: r.id.to_string(),
            source_document_id: r.source_document_id.to_string(),
            target_document_id: r.target_document_id.to_string(),
            relationship_type: r.relationship_type,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlDocumentFile {
    pub id: String,
    pub document_id: String,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub cdn_url: Option<String>,
    pub mime_type: String,
    pub file_size: String,
    pub uploaded_by: String,
    pub created_at: String,
}

impl From<crate::models::DocumentFile> for GqlDocumentFile {
    fn from(f: crate::models::DocumentFile) -> Self {
        Self {
            id: f.id.to_string(),
            document_id: f.document_id.to_string(),
            filename: f.filename,
            original_filename: f.original_filename,
            file_path: f.file_path,
            cdn_url: f.cdn_url,
            mime_type: f.mime_type,
            file_size: f.file_size.to_string(),
            uploaded_by: f.uploaded_by.to_string(),
            created_at: f.created_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlGlossaryEntry {
    pub id: String,
    pub term: String,
    pub slug: String,
    pub definition: String,
    pub extended_description: Option<String>,
    pub language: String,
    pub category_id: Option<String>,
    pub status: String,
    pub created_by: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::models::GlossaryEntry> for GqlGlossaryEntry {
    fn from(e: crate::models::GlossaryEntry) -> Self {
        Self {
            id: e.id.to_string(),
            term: e.term,
            slug: e.slug,
            definition: e.definition,
            extended_description: e.extended_description,
            language: e.language,
            category_id: e.category_id.map(|id| id.to_string()),
            status: e.status,
            created_by: e.created_by.to_string(),
            created_at: e.created_at.to_rfc3339(),
            updated_at: e.updated_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlGlossaryCategory {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl From<crate::models::GlossaryCategory> for GqlGlossaryCategory {
    fn from(c: crate::models::GlossaryCategory) -> Self {
        Self {
            id: c.id.to_string(),
            name: c.name,
            slug: c.slug,
            description: c.description,
            sort_order: c.sort_order,
            created_at: c.created_at.to_rfc3339(),
            updated_at: c.updated_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlGlossaryRelationship {
    pub id: String,
    pub source_entry_id: String,
    pub target_entry_id: String,
    pub relationship_type: String,
    pub created_at: String,
}

impl From<crate::models::GlossaryRelationship> for GqlGlossaryRelationship {
    fn from(r: crate::models::GlossaryRelationship) -> Self {
        Self {
            id: r.id.to_string(),
            source_entry_id: r.source_entry_id.to_string(),
            target_entry_id: r.target_entry_id.to_string(),
            relationship_type: r.relationship_type,
            created_at: r.created_at.to_rfc3339(),
        }
    }
}

#[derive(SimpleObject)]
pub struct GqlAuditLog {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub actor_id: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub changes: Option<String>,
    pub created_at: String,
}

impl From<crate::models::AuditLog> for GqlAuditLog {
    fn from(l: crate::models::AuditLog) -> Self {
        Self {
            id: l.id.to_string(),
            entity_type: l.entity_type,
            entity_id: l.entity_id.to_string(),
            action: l.action,
            actor_id: l.actor_id.to_string(),
            ip_address: l.ip_address,
            user_agent: l.user_agent,
            changes: l.changes.map(|c| c.to_string()),
            created_at: l.created_at.to_rfc3339(),
        }
    }
}

// ── Input Types ─────────────────────────────────────────────────────

#[derive(InputObject, Default)]
pub struct DocumentFilterInput {
    pub language: Option<String>,
    pub status: Option<String>,
    pub category_id: Option<ID>,
    pub created_by: Option<ID>,
}

#[derive(InputObject, Default)]
pub struct GlossaryFilterInput {
    pub language: Option<String>,
    pub category_id: Option<ID>,
    pub status: Option<String>,
}

#[derive(InputObject, Default)]
pub struct AuditLogFilterInput {
    pub entity_type: Option<String>,
    pub entity_id: Option<ID>,
    pub actor_id: Option<ID>,
}

#[derive(InputObject)]
pub struct PaginationInput {
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

// ── QueryRoot ───────────────────────────────────────────────────────

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn document(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GqlDocument>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&id)?;
        let doc = state.repos.documents().get_by_id(uuid).await?;
        Ok(doc.map(Into::into))
    }

    async fn documents(
        &self,
        ctx: &Context<'_>,
        filter: Option<DocumentFilterInput>,
        pagination: Option<PaginationInput>,
    ) -> async_graphql::Result<Vec<GqlDocument>> {
        let state = ctx.data::<AppState>()?;
        let f = filter.unwrap_or_default();
        let limit = pagination.as_ref().and_then(|p| p.limit).unwrap_or(50) as i64;
        let offset = pagination.as_ref().and_then(|p| p.offset).unwrap_or(0) as i64;
        let category_id = f
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());
        let created_by = f
            .created_by
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let docs = state
            .repos
            .documents()
            .list(
                category_id,
                f.status.as_deref(),
                f.language.as_deref(),
                created_by,
                limit,
                offset,
            )
            .await?;
        Ok(docs.into_iter().map(Into::into).collect())
    }

    async fn document_revisions(
        &self,
        ctx: &Context<'_>,
        document_id: ID,
    ) -> async_graphql::Result<Vec<GqlDocumentRevision>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&document_id)?;
        let revisions = state
            .repos
            .document_revisions()
            .get_by_document(uuid)
            .await?;
        Ok(revisions.into_iter().map(Into::into).collect())
    }

    async fn document_revision(
        &self,
        ctx: &Context<'_>,
        document_id: ID,
        revision_number: i32,
    ) -> async_graphql::Result<Option<GqlDocumentRevision>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&document_id)?;
        let rev = state
            .repos
            .document_revisions()
            .get_by_document_and_number(uuid, revision_number)
            .await?;
        Ok(rev.map(Into::into))
    }

    async fn document_relationships(
        &self,
        ctx: &Context<'_>,
        document_id: ID,
    ) -> async_graphql::Result<Vec<GqlDocumentRelationship>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&document_id)?;
        let rels = state
            .repos
            .document_relationships()
            .get_by_document(uuid)
            .await?;
        Ok(rels.into_iter().map(Into::into).collect())
    }

    async fn document_categories(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<GqlDocumentCategory>> {
        let state = ctx.data::<AppState>()?;
        let cats = state.repos.document_categories().list().await?;
        Ok(cats.into_iter().map(Into::into).collect())
    }

    async fn document_category(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GqlDocumentCategory>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&id)?;
        let cat = state.repos.document_categories().get_by_id(uuid).await?;
        Ok(cat.map(Into::into))
    }

    async fn document_files(
        &self,
        ctx: &Context<'_>,
        document_id: ID,
    ) -> async_graphql::Result<Vec<GqlDocumentFile>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&document_id)?;
        let files = state.repos.document_files().get_by_document(uuid).await?;
        Ok(files.into_iter().map(Into::into).collect())
    }

    async fn file(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GqlDocumentFile>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&id)?;
        let file = state.repos.document_files().get_by_id(uuid).await?;
        Ok(file.map(Into::into))
    }

    async fn glossary_entry(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&id)?;
        let entry = state.repos.glossary_entries().get_by_id(uuid).await?;
        Ok(entry.map(Into::into))
    }

    async fn glossary_entries(
        &self,
        ctx: &Context<'_>,
        filter: Option<GlossaryFilterInput>,
        pagination: Option<PaginationInput>,
    ) -> async_graphql::Result<Vec<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let f = filter.unwrap_or_default();
        let limit = pagination.as_ref().and_then(|p| p.limit).unwrap_or(50) as i64;
        let offset = pagination.as_ref().and_then(|p| p.offset).unwrap_or(0) as i64;
        let category_id = f
            .category_id
            .as_ref()
            .and_then(|id| Uuid::parse_str(id).ok());

        let entries = state
            .repos
            .glossary_entries()
            .list(
                category_id,
                f.status.as_deref(),
                f.language.as_deref(),
                limit,
                offset,
            )
            .await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn search_glossary(
        &self,
        ctx: &Context<'_>,
        search_term: String,
        pagination: Option<PaginationInput>,
    ) -> async_graphql::Result<Vec<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let limit = pagination.as_ref().and_then(|p| p.limit).unwrap_or(50) as i64;
        let offset = pagination.as_ref().and_then(|p| p.offset).unwrap_or(0) as i64;
        let entries = state
            .repos
            .glossary_entries()
            .search(&search_term, limit, offset)
            .await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn glossary_entry_relationships(
        &self,
        ctx: &Context<'_>,
        entry_id: ID,
    ) -> async_graphql::Result<Vec<GqlGlossaryRelationship>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&entry_id)?;
        let rels = state
            .repos
            .glossary_relationships()
            .get_by_entry(uuid)
            .await?;
        Ok(rels.into_iter().map(Into::into).collect())
    }

    async fn related_glossary_entries(
        &self,
        ctx: &Context<'_>,
        entry_id: ID,
    ) -> async_graphql::Result<Vec<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&entry_id)?;
        let entries = state.repos.glossary_entries().get_related(uuid).await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn glossary_categories(
        &self,
        ctx: &Context<'_>,
    ) -> async_graphql::Result<Vec<GqlGlossaryCategory>> {
        let state = ctx.data::<AppState>()?;
        let cats = state.repos.glossary_categories().list().await?;
        Ok(cats.into_iter().map(Into::into).collect())
    }

    async fn glossary_category(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> async_graphql::Result<Option<GqlGlossaryCategory>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&id)?;
        let cat = state.repos.glossary_categories().get_by_id(uuid).await?;
        Ok(cat.map(Into::into))
    }

    async fn glossary_entries_by_category(
        &self,
        ctx: &Context<'_>,
        category_id: ID,
    ) -> async_graphql::Result<Vec<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let uuid = Uuid::parse_str(&category_id)?;
        let entries = state.repos.glossary_entries().get_by_category(uuid).await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn glossary_entries_by_language(
        &self,
        ctx: &Context<'_>,
        language: String,
    ) -> async_graphql::Result<Vec<GqlGlossaryEntry>> {
        let state = ctx.data::<AppState>()?;
        let entries = state
            .repos
            .glossary_entries()
            .get_by_language(&language)
            .await?;
        Ok(entries.into_iter().map(Into::into).collect())
    }

    async fn audit_logs(
        &self,
        ctx: &Context<'_>,
        filter: Option<AuditLogFilterInput>,
        pagination: Option<PaginationInput>,
    ) -> async_graphql::Result<Vec<GqlAuditLog>> {
        let state = ctx.data::<AppState>()?;

        // Require manage permission for audit logs
        if let Ok(subject) = ctx.data::<crate::middleware::SubjectContext>() {
            state
                .keto
                .require_permission("document", "*", "manage", &subject.subject_id)
                .await?;
        }

        let f = filter.unwrap_or_default();
        let limit = pagination.as_ref().and_then(|p| p.limit).unwrap_or(50) as i64;
        let offset = pagination.as_ref().and_then(|p| p.offset).unwrap_or(0) as i64;
        let entity_id = f.entity_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());
        let actor_id = f.actor_id.as_ref().and_then(|id| Uuid::parse_str(id).ok());

        let logs = state
            .repos
            .audit()
            .get_audit_logs(f.entity_type.as_deref(), entity_id, actor_id, limit, offset)
            .await?;
        Ok(logs.into_iter().map(Into::into).collect())
    }

    async fn health(&self) -> String {
        "ok".to_string()
    }

    async fn system_info(&self, ctx: &Context<'_>) -> async_graphql::Result<serde_json::Value> {
        let state = ctx.data::<AppState>()?;

        if let Ok(subject) = ctx.data::<crate::middleware::SubjectContext>() {
            state
                .keto
                .require_permission("document", "*", "manage", &subject.subject_id)
                .await?;
        }

        Ok(serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "service": "adeptus",
            "nats_connected": state.events.is_connected(),
            "keto_configured": state.keto.is_configured(),
            "pactum_configured": state.pactum.is_configured(),
        }))
    }
}
