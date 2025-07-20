-- Rollback last_heartbeat field
ALTER TABLE Invocations DROP COLUMN last_heartbeat;
