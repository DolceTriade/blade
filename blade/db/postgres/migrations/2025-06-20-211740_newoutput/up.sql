-- Your SQL goes here

CREATE TABLE InvocationOutput (
    id SERIAL PRIMARY KEY,
    invocation_id TEXT NOT NULL REFERENCES Invocations(id) ON DELETE CASCADE,
    line TEXT NOT NULL
);

INSERT INTO InvocationOutput (invocation_id, line)
SELECT
    i.id,
    UNNEST(STRING_TO_ARRAY(i.output, E'\n')) AS line
FROM
    Invocations i;

ALTER TABLE Invocations
DROP COLUMN output;
