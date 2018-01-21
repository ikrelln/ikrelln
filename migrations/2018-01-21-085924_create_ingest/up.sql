CREATE TABLE ingest (
  id VARCHAR PRIMARY KEY,
  created_at TEXT NOT NULL,
  processed_at TEXT
);

CREATE TABLE test (
    id VARCHAR PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE test_result (
    id VARCHAR PRIMARY KEY,
    test_id VARCHAR NOT NULL,
    result VARCHAR NOT NULL,
    duration BIGINT NOT NULL,
    ts BIGINT NOT NULL
);
