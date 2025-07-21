-- Add last_heartbeat field for tracking active streams
ALTER TABLE invocations ADD COLUMN last_heartbeat TIMESTAMP;
