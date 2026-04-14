CREATE TABLE document_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
    revision_number INT NOT NULL,
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    change_summary TEXT,
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(document_id, revision_number)
);

CREATE INDEX idx_document_revisions_document_id ON document_revisions(document_id);
