CREATE TABLE document_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    parent_id UUID REFERENCES document_categories(id) ON DELETE SET NULL,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_document_categories_updated_at
    BEFORE UPDATE ON document_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Default categories
INSERT INTO document_categories (name, slug, description, sort_order) VALUES
    ('General', 'general', 'General documentation', 0),
    ('Technical', 'technical', 'Technical documentation', 1),
    ('Legal', 'legal', 'Legal documents', 2),
    ('Policy', 'policy', 'Company policies', 3),
    ('Training', 'training', 'Training materials', 4);
