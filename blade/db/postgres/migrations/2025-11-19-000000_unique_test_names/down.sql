-- Rollback: Drop trigger and function
DROP TRIGGER IF EXISTS maintain_unique_test_names_trigger ON tests;
DROP FUNCTION IF EXISTS maintain_unique_test_names();

-- Drop table
DROP TABLE IF EXISTS unique_test_names;
