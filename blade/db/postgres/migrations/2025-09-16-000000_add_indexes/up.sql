-- Add performance indexes based on observed query patterns
-- NOTE: existing indexes: Targets(invocation_id), Tests(invocation_id), Options(invocation_id), invocationoutput(invocation_id)
-- We avoid duplicates.

-- 1. Frequently ordered & ranged by start time (test history, pruning old invocations)
CREATE INDEX IF NOT EXISTS invocations_start_idx ON invocations (start DESC);

-- 2. Retrieval of test history filters by tests.name = $1 then joins invocations and orders by invocations.start
-- A btree index on name accelerates equality lookups.
CREATE INDEX IF NOT EXISTS tests_name_idx ON tests (name);

-- 3. TestRuns always accessed via belonging_to(tests) i.e. test_id equality
CREATE INDEX IF NOT EXISTS testruns_test_id_idx ON testruns (test_id);

-- 4. TestArtifacts often gathered per invocation (filter invocation_id = $1)
CREATE INDEX IF NOT EXISTS testartifacts_inv_id_idx ON testartifacts (invocation_id);

-- 5. Options filtered by (kind='Build Metadata') AND keyval eq/like patterns for metadata & flag queries.
-- Separate partial index for Build Metadata equality; general keyval index for flag lookups with prefix patterns.
CREATE INDEX IF NOT EXISTS options_keyval_idx ON options (keyval);
CREATE INDEX IF NOT EXISTS options_build_metadata_keyval_idx ON options (keyval) WHERE kind = 'Build Metadata';

-- 6. InvocationOutput: fetch lines by invocation ordered by id asc; deletes last N lines by ordering id desc.
-- Existing single-column index on invocation_id helps filter but not ordering; composite improves order + limit scans.
CREATE INDEX IF NOT EXISTS invocationoutput_inv_id_id_idx ON invocationoutput (invocation_id, id DESC);
