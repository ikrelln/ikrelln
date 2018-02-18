CREATE TABLE script
(
    id VARCHAR(36) NOT NULL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    source VARCHAR NOT NULL,
    script_type INT NOT NULL,
    date_added TIMESTAMP NOT NULL,
    status INT NOT NULL
);
