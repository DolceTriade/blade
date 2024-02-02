CREATE TABLE Options (
    id TEXT NOT NULL PRIMARY KEY,
    invocation_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    keyval TEXT NOT NULL,
    FOREIGN KEY(invocation_id) REFERENCES Invocations(id)
        ON DELETE CASCADE
);
CREATE INDEX Options_Inv_ID ON Options ( invocation_id );