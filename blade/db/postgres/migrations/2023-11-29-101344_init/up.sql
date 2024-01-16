CREATE TABLE Invocations (
	id TEXT NOT NULL PRIMARY KEY,
   	status TEXT NOT NULL,
	start TIMESTAMP WITH TIME ZONE NOT NULL,
    "end" TIMESTAMP WITH TIME ZONE,
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
    start TIMESTAMP WITH TIME ZONE NOT NULL,
    "end" TIMESTAMP WITH TIME ZONE,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);
CREATE INDEX Targets_Inv_ID ON Targets ( invocation_id );

CREATE TABLE Tests (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    duration_s double precision,
    "end" TIMESTAMP WITH TIME ZONE NOT NULL,
    num_runs INTEGER,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);
CREATE INDEX Tests_Inv_ID ON Tests ( invocation_id );

CREATE TABLE TestRuns (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    test_id TEXT NOT NULL,
    run INTEGER NOT NULL,
    shard INTEGER NOT NULL,
    attempt INTEGER NOT NULL,
    status TEXT NOT NULL,
    details TEXT NOT NULL,
    duration_s double precision NOT NULL,
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
