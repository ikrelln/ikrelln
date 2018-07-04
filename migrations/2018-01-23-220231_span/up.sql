CREATE TABLE endpoint
(
    endpoint_id VARCHAR(36) NOT NULL PRIMARY KEY,
    service_name VARCHAR(255),
    ipv4 VARCHAR(30),
    ipv6 VARCHAR(50),
    port INT,
    CONSTRAINT UC_Endpoint UNIQUE (service_name, ipv4, ipv6, port)
);
CREATE UNIQUE INDEX UC_Endpoint_name ON endpoint (service_name) WHERE (ipv4 is null AND ipv6 is null and port is null);
CREATE UNIQUE INDEX UC_Endpoint_name_ipv4 ON endpoint (service_name, ipv4) WHERE (ipv4 is not null AND ipv6 is null and port is null);
CREATE TABLE span
(
    trace_id VARCHAR(36) NOT NULL,
    id VARCHAR(36) NOT NULL UNIQUE,
    parent_id VARCHAR(36),
    name VARCHAR,
    kind VARCHAR(10),
    duration BIGINT,
    ts TIMESTAMP,
    debug BOOLEAN NOT NULL,
    shared BOOLEAN NOT NULL,
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
    ts TIMESTAMP NOT NULL,
    value VARCHAR(255) NOT NULL,
    FOREIGN KEY (trace_id, span_id) REFERENCES span (trace_id, id)
);
CREATE TABLE tag
(
    span_id VARCHAR(36) NOT NULL,
    name VARCHAR(255) NOT NULL,
    value VARCHAR NOT NULL,
    PRIMARY KEY (span_id, name),
    FOREIGN KEY (span_id) REFERENCES span (id)
);
