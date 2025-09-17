-- Recreate original btree indexes (may fail again if keyval entries exceed page limits)
-- Provided for symmetry; consider leaving dropped in production if values remain large.
DROP INDEX IF EXISTS options_keyval_prefix_idx;
DROP INDEX IF EXISTS options_build_metadata_keyval_hash_idx;
DROP INDEX IF EXISTS options_keyval_hash_idx;

-- (Re-)create original indexes
CREATE INDEX IF NOT EXISTS options_keyval_idx ON options (keyval);
CREATE INDEX IF NOT EXISTS options_build_metadata_keyval_idx ON options (keyval) WHERE kind = 'Build Metadata';
