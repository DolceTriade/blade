-- Rollback: Drop triggers and table
DROP TRIGGER IF EXISTS maintain_unique_test_names_insert;
DROP TRIGGER IF EXISTS maintain_unique_test_names_update;
DROP TRIGGER IF EXISTS maintain_unique_test_names_delete;

-- Drop table
DROP TABLE IF EXISTS unique_test_names;
