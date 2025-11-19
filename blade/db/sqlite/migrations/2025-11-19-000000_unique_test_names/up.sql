-- Create unique_test_names table to optimize autocomplete searches
CREATE TABLE IF NOT EXISTS unique_test_names (
    name TEXT NOT NULL PRIMARY KEY
);

-- Create index on name for LIKE '%pattern%' searches
CREATE INDEX IF NOT EXISTS unique_test_names_idx ON unique_test_names (name);

-- Seed the table with existing test names from tests table
INSERT OR IGNORE INTO unique_test_names (name)
SELECT DISTINCT name FROM tests;

-- Create trigger function to maintain unique_test_names when tests table changes
-- SQLite uses INSTEAD OF triggers for insert/update/delete, but since we're triggering AFTER,
-- we need separate triggers for each operation.

-- Trigger on INSERT: add new test name if it doesn't exist
CREATE TRIGGER IF NOT EXISTS maintain_unique_test_names_insert
AFTER INSERT ON tests
FOR EACH ROW
WHEN NEW.name NOT IN (SELECT name FROM unique_test_names)
BEGIN
    INSERT INTO unique_test_names (name)
    VALUES (NEW.name);
END;

-- Trigger on UPDATE: add updated test name if it doesn't exist
CREATE TRIGGER IF NOT EXISTS maintain_unique_test_names_update
AFTER UPDATE ON tests
FOR EACH ROW
WHEN NEW.name NOT IN (SELECT name FROM unique_test_names)
BEGIN
    INSERT INTO unique_test_names (name)
    VALUES (NEW.name);
END;

-- Trigger on DELETE: remove test name only if no other tests have this name
CREATE TRIGGER IF NOT EXISTS maintain_unique_test_names_delete
AFTER DELETE ON tests
FOR EACH ROW
BEGIN
    DELETE FROM unique_test_names
    WHERE name = OLD.name
    AND NOT EXISTS (SELECT 1 FROM tests WHERE name = OLD.name);
END;
