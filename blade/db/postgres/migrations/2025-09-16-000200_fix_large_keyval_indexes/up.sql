-- Mitigate oversized btree index tuples on options.keyval
-- Strategy:
-- 1. Drop problematic wide btree indexes (options_keyval_idx, options_build_metadata_keyval_idx)
-- 2. Add hash index for equality (fast, constant-length hash values)
-- 3. Add functional btree index on left(keyval, 200) to support prefix lookups and reduce tuple size
-- 4. Retain trigram GIN (if created) for ILIKE/substring searches
-- 5. Add partial hash index for Build Metadata equality queries (smaller, improves branch/commit lookups)
-- Notes:
-- * Hash indexes are WAL-logged and crash-safe since PG 10; acceptable here.
-- * left(keyval, 200) assumes relevant distinguishing prefix within first 200 chars. Adjust if needed.
-- * For LIKE 'foo%' queries planner can use btree(left(keyval,200)) if foo length <= 200.
-- * For exact equality, planner can choose hash index or trigram (usually hash).

-- Drop old wide btree indexes if they exist (avoid errors if already failed creation)
DROP INDEX IF EXISTS options_build_metadata_keyval_idx;
DROP INDEX IF EXISTS options_keyval_idx;

-- Ensure pg_trgm extension is present if relying on trigram search (no-op if exists)
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Equality hash index on full keyval
CREATE INDEX IF NOT EXISTS options_keyval_hash_idx ON options USING HASH (keyval);

-- Partial equality hash index for build metadata subset
CREATE INDEX IF NOT EXISTS options_build_metadata_keyval_hash_idx ON options USING HASH (keyval) WHERE kind='Build Metadata';

-- Prefix btree index on first 200 characters (adjust length if needed)
CREATE INDEX IF NOT EXISTS options_keyval_prefix_idx ON options (left(keyval, 200));
