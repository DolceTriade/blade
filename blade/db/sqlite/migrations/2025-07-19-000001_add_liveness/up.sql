-- Add last_heartbeat field for tracking active streams
ALTER TABLE Invocations ADD COLUMN last_heartbeat TEXT;
