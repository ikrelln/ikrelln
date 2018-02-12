CREATE TABLE test
(
    id VARCHAR NOT NULL PRIMARY KEY,
    test_suite VARCHAR(255) NOT NULL,
    test_class VARCHAR(255) NOT NULL,
    test_name VARCHAR(255) NOT NULL,
    CONSTRAINT UC_Test UNIQUE (test_suite, test_class, test_name)
);
CREATE TABLE test_execution
(
    test_id VARCHAR NOT NULL,
    trace_id VARCHAR(36) NOT NULL,
    date TIMESTAMP NOT NULL,
    result INT NOT NULL,
    duration BIGINT NOT NULL,
    environment VARCHAR,
    PRIMARY KEY (test_id, trace_id),
    FOREIGN KEY (test_id) REFERENCES test(id)
);
