use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub content: String,
    pub language: String,
    pub category_id: Option<Uuid>,
    pub tags: Vec<String>,
    pub status: String,
    pub published_at: Option<DateTime<Utc>>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRevision {
    pub id: Uuid,
    pub document_id: Uuid,
    pub revision_number: i32,
    pub title: String,
    pub content: String,
    pub change_summary: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRelationship {
    pub id: Uuid,
    pub source_document_id: Uuid,
    pub target_document_id: Uuid,
    pub relationship_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentFile {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryCategory {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntry {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryRelationship {
    pub id: Uuid,
    pub source_entry_id: Uuid,
    pub target_entry_id: Uuid,
    pub relationship_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_serde_roundtrip() {
        let doc = Document {
            id: Uuid::new_v4(),
            title: "Test".to_string(),
            slug: "test".to_string(),
            description: Some("A test document".to_string()),
            content: "# Hello".to_string(),
            language: "en".to_string(),
            category_id: None,
            tags: vec!["test".to_string()],
            status: "draft".to_string(),
            published_at: None,
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&doc).unwrap();
        let deserialized: Document = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, doc.title);
        assert_eq!(deserialized.tags, doc.tags);
    }

    #[test]
    fn test_glossary_entry_serde_roundtrip() {
        let entry = GlossaryEntry {
            id: Uuid::new_v4(),
            term: "API".to_string(),
            slug: "api".to_string(),
            definition: "Application Programming Interface".to_string(),
            extended_description: None,
            language: "en".to_string(),
            category_id: None,
            status: "published".to_string(),
            created_by: Uuid::new_v4(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: GlossaryEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.term, entry.term);
    }

    #[test]
    fn test_audit_log_serde_roundtrip() {
        let log = AuditLog {
            id: Uuid::new_v4(),
            entity_type: "document".to_string(),
            entity_id: Uuid::new_v4(),
            action: "created".to_string(),
            actor_id: Uuid::new_v4(),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("Test/1.0".to_string()),
            changes: Some(serde_json::json!({"title": "New Title"})),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&log).unwrap();
        let deserialized: AuditLog = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.entity_type, log.entity_type);
    }
}
