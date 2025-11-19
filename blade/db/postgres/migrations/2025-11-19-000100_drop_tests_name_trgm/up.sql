-- Drop trigram index on tests.name since we now search on unique_test_names table
-- Keep the regular btree index (tests_name_idx) for joins and equality filters
DROP INDEX IF EXISTS tests_name_trgm_idx;
