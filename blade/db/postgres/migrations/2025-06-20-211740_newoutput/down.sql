-- This file should undo anything in `up.sql`
-- Migration to Undo Previous Changes

-- Step 1: Add the 'output' column back to Invocations
ALTER TABLE Invocations
ADD COLUMN output TEXT NOT NULL DEFAULT ''; -- Add NOT NULL and a default empty string for new rows

-- Important Note for Step 1:
-- If your 'output' column previously allowed NULLs, adjust the above line:
-- ALTER TABLE Invocations
-- ADD COLUMN output TEXT;
-- You might also want to set a default value for existing rows that will get NULL initially.
-- For simplicity and assuming previous 'output' was NOT NULL, we use NOT NULL DEFAULT ''.

-- Step 2: Migrate data back from InvocationOutput to Invocations
-- This aggregates the lines back into a single text block, preserving order.
UPDATE Invocations inv
SET output = (
    SELECT STRING_AGG(io.line, E'\n' ORDER BY io.id)
    FROM InvocationOutput io
    WHERE io.invocation_id = inv.id
)
WHERE EXISTS (SELECT 1 FROM InvocationOutput io WHERE io.invocation_id = inv.id);

-- Handle cases where an invocation might have had no output (if original 'output' could be empty/null)
-- If an invocation had no entries in InvocationOutput, its 'output' column would remain ''.
-- If you need to revert to NULL for such cases, you might do:
-- UPDATE Invocations SET output = NULL WHERE output = ''; -- Only if 'output' was nullable

-- Step 3: Drop the InvocationOutput table
DROP TABLE IF EXISTS InvocationOutput;
