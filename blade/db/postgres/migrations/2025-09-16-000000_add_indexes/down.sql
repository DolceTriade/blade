-- Drop performance indexes
DROP INDEX IF EXISTS invocationoutput_inv_id_id_idx;
DROP INDEX IF EXISTS options_build_metadata_keyval_idx;
DROP INDEX IF EXISTS options_keyval_idx;
DROP INDEX IF EXISTS testartifacts_inv_id_idx;
DROP INDEX IF EXISTS testruns_test_id_idx;
DROP INDEX IF EXISTS tests_name_idx;
DROP INDEX IF EXISTS invocations_start_idx;
