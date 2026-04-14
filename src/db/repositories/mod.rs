pub mod audit;
pub mod document;
pub mod document_category;
pub mod document_file;
pub mod document_relationship;
pub mod document_revision;
pub mod glossary_category;
pub mod glossary_entry;
pub mod glossary_relationship;

pub use audit::AuditRepository;
pub use document::DocumentRepository;
pub use document_category::DocumentCategoryRepository;
pub use document_file::DocumentFileRepository;
pub use document_relationship::DocumentRelationshipRepository;
pub use document_revision::DocumentRevisionRepository;
pub use glossary_category::GlossaryCategoryRepository;
pub use glossary_entry::GlossaryEntryRepository;
pub use glossary_relationship::GlossaryRelationshipRepository;

use sqlx::PgPool;

#[derive(Clone)]
pub struct RepositoryManager {
    pool: PgPool,
}

impl RepositoryManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn documents(&self) -> DocumentRepository {
        DocumentRepository::new(self.pool.clone())
    }

    pub fn document_categories(&self) -> DocumentCategoryRepository {
        DocumentCategoryRepository::new(self.pool.clone())
    }

    pub fn document_revisions(&self) -> DocumentRevisionRepository {
        DocumentRevisionRepository::new(self.pool.clone())
    }

    pub fn document_relationships(&self) -> DocumentRelationshipRepository {
        DocumentRelationshipRepository::new(self.pool.clone())
    }

    pub fn document_files(&self) -> DocumentFileRepository {
        DocumentFileRepository::new(self.pool.clone())
    }

    pub fn glossary_entries(&self) -> GlossaryEntryRepository {
        GlossaryEntryRepository::new(self.pool.clone())
    }

    pub fn glossary_categories(&self) -> GlossaryCategoryRepository {
        GlossaryCategoryRepository::new(self.pool.clone())
    }

    pub fn glossary_relationships(&self) -> GlossaryRelationshipRepository {
        GlossaryRelationshipRepository::new(self.pool.clone())
    }

    pub fn audit(&self) -> AuditRepository {
        AuditRepository::new(self.pool.clone())
    }
}
