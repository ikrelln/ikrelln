CREATE TABLE report
(
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR NOT NULL,
    folder VARCHAR NOT NULL,
    created_on TIMESTAMP NOT NULL,
    last_update TIMESTAMP NOT NULL,
    CONSTRAINT UC_Report UNIQUE (folder, name)
);
CREATE TABLE test_result_in_report
(
    report_id VARCHAR(36) NOT NULL,
    test_id VARCHAR(36) NOT NULL,
    trace_id VARCHAR(36) NOT NULL,
    category VARCHAR NOT NULL,
    environment VARCHAR,
    status INT NOT NULL,
    PRIMARY KEY (report_id, test_id, trace_id, category),
    FOREIGN KEY (report_id) REFERENCES report(id),
    FOREIGN KEY (test_id, trace_id) REFERENCES test_result (test_id, trace_id)
);
