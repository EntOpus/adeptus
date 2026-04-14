CREATE TABLE document_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    target_document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_document_id, target_document_id, relationship_type)
);

CREATE INDEX idx_document_relationships_source ON document_relationships(source_document_id);
CREATE INDEX idx_document_relationships_target ON document_relationships(target_document_id);
