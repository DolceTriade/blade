-- Drop trigram indexes then extension (extension drop will fail if others depend on it)
DROP INDEX IF EXISTS options_keyval_trgm_idx;
DROP INDEX IF EXISTS invocationoutput_line_trgm_idx;
DROP INDEX IF EXISTS tests_name_trgm_idx;
DROP EXTENSION IF EXISTS pg_trgm;
