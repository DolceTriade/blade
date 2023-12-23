-- Your SQL goes here
CREATE TABLE Invocations (
	id TEXT PRIMARY KEY,
   	status TEXT NOT NULL,
	start TIMESTAMP NOT NULL,
    output TEXT NOT NULL,
    command TEXT NOT NULL,
    pattern TEXT
);

CREATE TABLE Targets (
    id TEXT PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    kind TEXT NOT NULL,
    start TIMESTAMP NOT NULL,
    end TIMESTAMP,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);

CREATE TABLE Tests (
    id TEXT PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_s REAL,
    num_runs INTEGER,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);

CREATE TABLE TestRuns (
    id TEXT PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    test_id TEXT NOT NULL,
    run INTEGER NOT NULL,
    shard INTEGER NOT NULL,
    attempt INTEGER NOT NULL,
    duration_s REAL NOT NULL,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE,
    FOREIGN KEY(test_id) REFERENCES Tests(id)
        ON DELETE CASCADE
);

CREATE TABLE TestArtifacts (
    id TEXT PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    test_run_id TEXT NOT NULL,
    uri TEXT NOT NULL,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE,
    FOREIGN KEY(test_run_id) REFERENCES TestRuns(id)
        ON DELETE CASCADE
);
