CREATE INDEX IF NOT EXISTS invocationoutput_inv_id ON invocationoutput (invocation_id);

-- For SQLite:
-- CREATE INDEX IF NOT EXISTS idx_invocationoutput_invocation_id ON invocationoutput (invocation_id);

-- For MySQL:
-- CREATE INDEX idx_invocationoutput_invocation_id ON invocationoutput (invocation_id);
-- MySQL's CREATE INDEX does not have IF NOT EXISTS; it will error if the index exists.
-- You typically rely on the migration system to ensure it's only run once.
