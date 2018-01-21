CREATE TABLE ingest (
  id VARCHAR PRIMARY KEY,
  created_at TEXT NOT NULL,
  processed_at TEXT
);

CREATE TABLE ingest_events (
    id VARCHAR PRIMARY KEY,
    ingest_id VARCHAR NOT NULL,
    event_type VARCHAR NOT NULL
);
