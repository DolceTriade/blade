-- Your SQL goes here
CREATE TABLE Invocations (
	id TEXT NOT NULL PRIMARY KEY,
   	status TEXT NOT NULL,
	start TEXT NOT NULL,
    output TEXT NOT NULL,
    command TEXT NOT NULL,
    pattern TEXT
);

CREATE TABLE Targets (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    kind TEXT NOT NULL,
    start TEXT NOT NULL,
    end TEXT,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);

CREATE TABLE Tests (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_s Double,
    num_runs INTEGER,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);

CREATE TABLE TestRuns (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    test_id TEXT NOT NULL,
    run INTEGER NOT NULL,
    shard INTEGER NOT NULL,
    attempt INTEGER NOT NULL,
    status TEXT NOT NULL,
    details TEXT NOT NULL,
    duration_s Double NOT NULL,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE,
    FOREIGN KEY(test_id) REFERENCES Tests(id)
        ON DELETE CASCADE
);

CREATE TABLE TestArtifacts (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    test_run_id TEXT NOT NULL,
    name TEXT NOT NULL,
    uri TEXT NOT NULL,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE,
    FOREIGN KEY(test_run_id) REFERENCES TestRuns(id)
        ON DELETE CASCADE
);
