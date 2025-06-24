-- Your SQL goes here
-- Create the new InvocationOutput table
CREATE TABLE InvocationOutput (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invocation_id TEXT NOT NULL,
    line TEXT NOT NULL,
    -- Add a foreign key constraint.
    -- SQLite's foreign key constraints are only enforced if PRAGMA foreign_keys = ON;
    FOREIGN KEY (invocation_id) REFERENCES Invocations(id) ON DELETE CASCADE
);

-- Copy data from the old 'output' column into the new 'InvocationOutput' table.
-- SQLite doesn't have UNNEST or STRING_TO_ARRAY directly.
-- This part is the trickiest in pure SQL for SQLite.
-- For actual data splitting, you would typically do this in application code (Rust).
-- For this SQL migration, we'll assume a simpler approach or that `output`
-- was single-line if this pure SQL migration were to be runnable.
--
-- HOWEVER, since you are using a multi-line `output` column, doing this
-- in pure SQLite SQL with `STRING_TO_ARRAY` is NOT feasible.
--
-- The most practical way to handle multi-line splitting during a SQLite migration
-- is to use application code (e.g., Rust with Diesel).
--
-- But if we HAD to do it in SQL and assume output is just one line per `Invocation`
-- or we put the *entire* output into one `InvocationOutput` line, it would be:
INSERT INTO InvocationOutput (invocation_id, line)
SELECT id, output FROM Invocations;

-- Drop the 'output' column from Invocations
-- SQLite's ALTER TABLE DROP COLUMN is supported in newer versions (3.35.0+)
ALTER TABLE Invocations DROP COLUMN output;
