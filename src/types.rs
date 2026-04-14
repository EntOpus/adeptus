use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    Draft,
    Published,
    Unpublished,
    Archived,
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentStatus::Draft => write!(f, "draft"),
            DocumentStatus::Published => write!(f, "published"),
            DocumentStatus::Unpublished => write!(f, "unpublished"),
            DocumentStatus::Archived => write!(f, "archived"),
        }
    }
}

impl std::str::FromStr for DocumentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(DocumentStatus::Draft),
            "published" => Ok(DocumentStatus::Published),
            "unpublished" => Ok(DocumentStatus::Unpublished),
            "archived" => Ok(DocumentStatus::Archived),
            _ => Err(format!("unknown document status: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipType {
    RelatedTo,
    DependsOn,
    Supersedes,
    References,
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipType::RelatedTo => write!(f, "related_to"),
            RelationshipType::DependsOn => write!(f, "depends_on"),
            RelationshipType::Supersedes => write!(f, "supersedes"),
            RelationshipType::References => write!(f, "references"),
        }
    }
}

impl std::str::FromStr for RelationshipType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "related_to" => Ok(RelationshipType::RelatedTo),
            "depends_on" => Ok(RelationshipType::DependsOn),
            "supersedes" => Ok(RelationshipType::Supersedes),
            "references" => Ok(RelationshipType::References),
            _ => Err(format!("unknown relationship type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryRelationshipType {
    Synonym,
    Antonym,
    RelatedTo,
    SeeAlso,
    ParentOf,
    ChildOf,
}

impl std::fmt::Display for GlossaryRelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GlossaryRelationshipType::Synonym => write!(f, "synonym"),
            GlossaryRelationshipType::Antonym => write!(f, "antonym"),
            GlossaryRelationshipType::RelatedTo => write!(f, "related_to"),
            GlossaryRelationshipType::SeeAlso => write!(f, "see_also"),
            GlossaryRelationshipType::ParentOf => write!(f, "parent_of"),
            GlossaryRelationshipType::ChildOf => write!(f, "child_of"),
        }
    }
}

impl std::str::FromStr for GlossaryRelationshipType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "synonym" => Ok(GlossaryRelationshipType::Synonym),
            "antonym" => Ok(GlossaryRelationshipType::Antonym),
            "related_to" => Ok(GlossaryRelationshipType::RelatedTo),
            "see_also" => Ok(GlossaryRelationshipType::SeeAlso),
            "parent_of" => Ok(GlossaryRelationshipType::ParentOf),
            "child_of" => Ok(GlossaryRelationshipType::ChildOf),
            _ => Err(format!("unknown glossary relationship type: {s}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditAction {
    Created,
    Updated,
    Deleted,
    Published,
    Unpublished,
    Archived,
    FileUploaded,
    FileDeleted,
    PdfGenerated,
    RelationshipCreated,
    RelationshipDeleted,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Created => write!(f, "created"),
            AuditAction::Updated => write!(f, "updated"),
            AuditAction::Deleted => write!(f, "deleted"),
            AuditAction::Published => write!(f, "published"),
            AuditAction::Unpublished => write!(f, "unpublished"),
            AuditAction::Archived => write!(f, "archived"),
            AuditAction::FileUploaded => write!(f, "file_uploaded"),
            AuditAction::FileDeleted => write!(f, "file_deleted"),
            AuditAction::PdfGenerated => write!(f, "pdf_generated"),
            AuditAction::RelationshipCreated => write!(f, "relationship_created"),
            AuditAction::RelationshipDeleted => write!(f, "relationship_deleted"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Document,
    DocumentCategory,
    GlossaryEntry,
    GlossaryCategory,
    DocumentFile,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Document => write!(f, "document"),
            EntityType::DocumentCategory => write!(f, "document_category"),
            EntityType::GlossaryEntry => write!(f, "glossary_entry"),
            EntityType::GlossaryCategory => write!(f, "glossary_category"),
            EntityType::DocumentFile => write!(f, "document_file"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: i32,
    pub offset: i32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub has_more: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_status_display_and_parse() {
        let variants = [
            DocumentStatus::Draft,
            DocumentStatus::Published,
            DocumentStatus::Unpublished,
            DocumentStatus::Archived,
        ];
        for variant in &variants {
            let s = variant.to_string();
            assert_eq!(&s.parse::<DocumentStatus>().unwrap(), variant);
        }
    }

    #[test]
    fn test_relationship_type_display_and_parse() {
        let variants = [
            RelationshipType::RelatedTo,
            RelationshipType::DependsOn,
            RelationshipType::Supersedes,
            RelationshipType::References,
        ];
        for variant in &variants {
            let s = variant.to_string();
            assert_eq!(&s.parse::<RelationshipType>().unwrap(), variant);
        }
    }

    #[test]
    fn test_glossary_relationship_type_display_and_parse() {
        let variants = [
            GlossaryRelationshipType::Synonym,
            GlossaryRelationshipType::Antonym,
            GlossaryRelationshipType::RelatedTo,
            GlossaryRelationshipType::SeeAlso,
            GlossaryRelationshipType::ParentOf,
            GlossaryRelationshipType::ChildOf,
        ];
        for variant in &variants {
            let s = variant.to_string();
            assert_eq!(&s.parse::<GlossaryRelationshipType>().unwrap(), variant);
        }
    }

    #[test]
    fn test_audit_action_display() {
        assert_eq!(AuditAction::Created.to_string(), "created");
        assert_eq!(AuditAction::Published.to_string(), "published");
        assert_eq!(AuditAction::PdfGenerated.to_string(), "pdf_generated");
    }

    #[test]
    fn test_entity_type_display() {
        assert_eq!(EntityType::Document.to_string(), "document");
        assert_eq!(EntityType::GlossaryEntry.to_string(), "glossary_entry");
    }

    #[test]
    fn test_pagination_default() {
        let p = Pagination::default();
        assert_eq!(p.limit, 50);
        assert_eq!(p.offset, 0);
    }
}
