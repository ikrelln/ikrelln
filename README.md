# i'Krelln [![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0) [![Build Status](https://travis-ci.org/ikrelln/ikrelln.svg?branch=master)](https://travis-ci.org/ikrelln/ikrelln)
Test Reporting for the robots _because who has time to read all those tests results?_

i'Krelln is a test reporting and tracing system. It helps gather test execution data needed to troubleshoot failures. It manages both the collection and lookup of this data.

## Quick start

You can start i'Krelln by Docker:
 
```bash
docker run -d -p 7878:7878 -e DATABASE_URL=postgres://postgreshost:5432/ ikrelln/ikrelln
```

Once it started, you can send your spans on http://localhost:7878/api/v1/spans.

### All in one Docker image

This image, designed for quick local testing, launch i'Krelln, the zipkin ui and a postgres DB.

```bash
docker run -d -p 7878:7878 -p 9411:80 ikrelln/ikrelln:all-in-one
```

Once started, you can send your spans to http://localhost:7878/api/v1/spans, and access zipkin ui on http://localhost:9411/zipkin/
