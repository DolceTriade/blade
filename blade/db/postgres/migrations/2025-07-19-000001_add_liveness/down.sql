-- Rollback last_heartbeat field
ALTER TABLE invocations DROP COLUMN last_heartbeat;
