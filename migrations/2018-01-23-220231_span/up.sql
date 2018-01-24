CREATE TABLE span
(
    trace_id VARCHAR NOT NULL,
    parent_id VARCHAR,
    id VARCHAR NOT NULL PRIMARY KEY,
    name VARCHAR,
    duration BIGINT NOT NULL,
    ts BIGINT NOT NULL
);
