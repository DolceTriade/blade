-- Rollback: Recreate trigram index on tests.name
CREATE INDEX IF NOT EXISTS tests_name_trgm_idx ON tests USING GIN (name gin_trgm_ops);
