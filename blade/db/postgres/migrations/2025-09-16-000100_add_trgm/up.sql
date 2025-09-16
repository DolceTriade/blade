-- Enable trigram matching extension (safe to IF NOT EXISTS on modern PG versions >=13)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- GIN trigram indexes to accelerate substring / ILIKE searches:
-- 1. tests.name: used in LIKE '%pattern%' search in search_test_names.
CREATE INDEX IF NOT EXISTS tests_name_trgm_idx ON tests USING GIN (name gin_trgm_ops);

-- 2. invocationoutput.line: used for log output filters with ILIKE / Contains.
CREATE INDEX IF NOT EXISTS invocationoutput_line_trgm_idx ON invocationoutput USING GIN (line gin_trgm_ops);

-- 3. options.keyval: used for Bazel flag / metadata contains searches (LIKE / ILIKE patterns).
CREATE INDEX IF NOT EXISTS options_keyval_trgm_idx ON options USING GIN (keyval gin_trgm_ops);

-- Note: retain existing btree indexes for equality and ordering; GIN complements them for pattern scans.
