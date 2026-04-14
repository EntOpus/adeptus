CREATE TABLE glossary_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    sort_order INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER update_glossary_categories_updated_at
    BEFORE UPDATE ON glossary_categories
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Default glossary categories
INSERT INTO glossary_categories (name, slug, description, sort_order) VALUES
    ('General', 'general', 'General terms', 0),
    ('Technical', 'technical', 'Technical terminology', 1),
    ('Business', 'business', 'Business terminology', 2);
