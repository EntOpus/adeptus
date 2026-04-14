CREATE TABLE glossary_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    term VARCHAR(500) NOT NULL,
    slug VARCHAR(500) NOT NULL UNIQUE,
    definition TEXT NOT NULL,
    extended_description TEXT,
    language VARCHAR(10) NOT NULL DEFAULT 'en',
    category_id UUID REFERENCES glossary_categories(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    created_by UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_glossary_entries_category_id ON glossary_entries(category_id);
CREATE INDEX idx_glossary_entries_language ON glossary_entries(language);
CREATE INDEX idx_glossary_entries_status ON glossary_entries(status);
CREATE INDEX idx_glossary_entries_term ON glossary_entries(term);

CREATE TRIGGER update_glossary_entries_updated_at
    BEFORE UPDATE ON glossary_entries
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
