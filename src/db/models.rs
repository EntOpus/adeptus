use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct DocumentRow {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub content: String,
    pub language: String,
    pub category_id: Option<Uuid>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DocumentRow> for crate::models::Document {
    fn from(row: DocumentRow) -> Self {
        Self {
            id: row.id,
            title: row.title,
            slug: row.slug,
            description: row.description,
            content: row.content,
            language: row.language,
            category_id: row.category_id,
            tags: row.tags.unwrap_or_default(),
            status: row.status,
            published_at: row.published_at,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentCategoryRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DocumentCategoryRow> for crate::models::DocumentCategory {
    fn from(row: DocumentCategoryRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            parent_id: row.parent_id,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentRevisionRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub revision_number: i32,
    pub title: String,
    pub content: String,
    pub change_summary: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<DocumentRevisionRow> for crate::models::DocumentRevision {
    fn from(row: DocumentRevisionRow) -> Self {
        Self {
            id: row.id,
            document_id: row.document_id,
            revision_number: row.revision_number,
            title: row.title,
            content: row.content,
            change_summary: row.change_summary,
            created_by: row.created_by,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentRelationshipRow {
    pub id: Uuid,
    pub source_document_id: Uuid,
    pub target_document_id: Uuid,
    pub relationship_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<DocumentRelationshipRow> for crate::models::DocumentRelationship {
    fn from(row: DocumentRelationshipRow) -> Self {
        Self {
            id: row.id,
            source_document_id: row.source_document_id,
            target_document_id: row.target_document_id,
            relationship_type: row.relationship_type,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct DocumentFileRow {
    pub id: Uuid,
    pub document_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub file_path: String,
    pub cdn_url: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub uploaded_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<DocumentFileRow> for crate::models::DocumentFile {
    fn from(row: DocumentFileRow) -> Self {
        Self {
            id: row.id,
            document_id: row.document_id,
            filename: row.filename,
            original_filename: row.original_filename,
            file_path: row.file_path,
            cdn_url: row.cdn_url,
            mime_type: row.mime_type,
            file_size: row.file_size,
            uploaded_by: row.uploaded_by,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GlossaryCategoryRow {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<GlossaryCategoryRow> for crate::models::GlossaryCategory {
    fn from(row: GlossaryCategoryRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            slug: row.slug,
            description: row.description,
            sort_order: row.sort_order,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GlossaryEntryRow {
    pub id: Uuid,
    pub term: String,
    pub slug: String,
    pub definition: String,
    pub extended_description: Option<String>,
    pub language: String,
    pub category_id: Option<Uuid>,
    pub status: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<GlossaryEntryRow> for crate::models::GlossaryEntry {
    fn from(row: GlossaryEntryRow) -> Self {
        Self {
            id: row.id,
            term: row.term,
            slug: row.slug,
            definition: row.definition,
            extended_description: row.extended_description,
            language: row.language,
            category_id: row.category_id,
            status: row.status,
            created_by: row.created_by,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct GlossaryRelationshipRow {
    pub id: Uuid,
    pub source_entry_id: Uuid,
    pub target_entry_id: Uuid,
    pub relationship_type: String,
    pub created_at: DateTime<Utc>,
}

impl From<GlossaryRelationshipRow> for crate::models::GlossaryRelationship {
    fn from(row: GlossaryRelationshipRow) -> Self {
        Self {
            id: row.id,
            source_entry_id: row.source_entry_id,
            target_entry_id: row.target_entry_id,
            relationship_type: row.relationship_type,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub actor_id: Uuid,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub changes: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogRow> for crate::models::AuditLog {
    fn from(row: AuditLogRow) -> Self {
        Self {
            id: row.id,
            entity_type: row.entity_type,
            entity_id: row.entity_id,
            action: row.action,
            actor_id: row.actor_id,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            changes: row.changes,
            created_at: row.created_at,
        }
    }
}
