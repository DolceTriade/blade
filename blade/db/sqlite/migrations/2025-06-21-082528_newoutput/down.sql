-- This file should undo anything in `up.sql`
-- Step 1: Add the 'output' column back to Invocations
-- Note: SQLite does not support ADD COLUMN NOT NULL without a default value
-- for existing rows. If you want it NOT NULL, you usually need to recreate the table.
-- Assuming original 'output' was NOT NULL based on your schema.
-- This will add a nullable 'output' column initially.
ALTER TABLE Invocations ADD COLUMN output TEXT;

-- Step 2: Migrate data back from InvocationOutput to Invocations
-- This aggregates lines back into a single TEXT block, ordered by ID.
-- SQLite's GROUP_CONCAT is used for aggregation.
UPDATE Invocations
SET output = (
    SELECT GROUP_CONCAT(line, X'0A') -- X'0A' is the hex representation for newline character
    FROM InvocationOutput
    WHERE InvocationOutput.invocation_id = Invocations.id
    ORDER BY InvocationOutput.id -- Ensure order is preserved
);

-- Handle cases where an invocation might not have any output entries in InvocationOutput.
-- If you want NULL for these, do:
-- UPDATE Invocations SET output = NULL WHERE output IS NULL; -- (if you added it nullable)
-- Or if you want empty string (which is how ADD COLUMN TEXT without default would appear for existing data)
-- UPDATE Invocations SET output = '' WHERE output IS NULL;

-- Step 3: Drop the InvocationOutput table
DROP TABLE IF EXISTS InvocationOutput;
