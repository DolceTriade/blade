-- Create unique_test_names table to optimize autocomplete searches
CREATE TABLE IF NOT EXISTS unique_test_names (
    name TEXT NOT NULL PRIMARY KEY
);

-- Create GIN index on name for LIKE '%pattern%' searches
CREATE INDEX IF NOT EXISTS unique_test_names_trgm_idx ON unique_test_names USING GIN (name gin_trgm_ops);

-- Seed the table with existing test names from tests table
INSERT INTO unique_test_names (name)
SELECT DISTINCT name FROM tests
ON CONFLICT (name) DO NOTHING;

-- Create trigger function to maintain unique_test_names when tests table changes
CREATE OR REPLACE FUNCTION maintain_unique_test_names()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
        -- Insert new test name if it doesn't exist
        INSERT INTO unique_test_names (name)
        VALUES (NEW.name)
        ON CONFLICT (name) DO NOTHING;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        -- Remove test name only if no other tests have this name
        DELETE FROM unique_test_names
        WHERE name = OLD.name
        AND NOT EXISTS (SELECT 1 FROM tests WHERE name = OLD.name);
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create trigger on tests table to maintain unique_test_names
DROP TRIGGER IF EXISTS maintain_unique_test_names_trigger ON tests;
CREATE TRIGGER maintain_unique_test_names_trigger
AFTER INSERT OR UPDATE OR DELETE ON tests
FOR EACH ROW
EXECUTE FUNCTION maintain_unique_test_names();
