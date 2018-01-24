CREATE TABLE endpoint
(
    endpoint_id VARCHAR(36) NOT NULL PRIMARY KEY,
    service_name VARCHAR(255),
    ipv4 VARCHAR(30),
    ipv6 VARCHAR(50),
    port INT
);
CREATE TABLE span
(
    trace_id VARCHAR(36) NOT NULL,
    id VARCHAR(36) NOT NULL,
    parent_id VARCHAR(36),
    name VARCHAR(36),
    kind VARCHAR(10),
    duration BIGINT,
    ts BIGINT,
    debug BOOLEAN,
    shared BOOLEAN,
    local_endpoint_id VARCHAR(36),
    remote_endpoint_id VARCHAR(36),
    PRIMARY KEY (trace_id, id),
    FOREIGN KEY (local_endpoint_id) REFERENCES endpoint (endpoint_id),
    FOREIGN KEY (remote_endpoint_id) REFERENCES endpoint (endpoint_id)
);
CREATE TABLE annotation
(
    annotation_id VARCHAR(36) NOT NULL PRIMARY KEY,
    trace_id VARCHAR(36) NOT NULL,
    span_id VARCHAR(36) NOT NULL,
    FOREIGN KEY (trace_id, span_id) REFERENCES span (trace_id, id)
);
CREATE TABLE tag
(
    tag_id VARCHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);
CREATE TABLE tag_value
(
    tag_value_id VARCHAR(36) NOT NULL PRIMARY KEY,
    tag_id VARCHAR(36) NOT NULL,
    trace_id VARCHAR(36) NOT NULL,
    span_id VARCHAR(36) NOT NULL,
    value VARCHAR(255) NOT NULL,
    FOREIGN KEY (tag_id) REFERENCES tag (tag_id),
    FOREIGN KEY (trace_id, span_id) REFERENCES span (trace_id, id)
);
