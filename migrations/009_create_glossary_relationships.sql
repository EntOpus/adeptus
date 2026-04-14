CREATE TABLE glossary_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_entry_id UUID NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    target_entry_id UUID NOT NULL REFERENCES glossary_entries(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_entry_id, target_entry_id, relationship_type)
);

CREATE INDEX idx_glossary_relationships_source ON glossary_relationships(source_entry_id);
CREATE INDEX idx_glossary_relationships_target ON glossary_relationships(target_entry_id);
