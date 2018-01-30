# i'Krelln [![Build Status](https://travis-ci.org/ikrelln/ikrelln.svg?branch=master)](https://travis-ci.org/ikrelln/ikrelln)
Test Reporting for the robots _because who has time to read all those tests results?_

i'Krelln is a test reporting and tracing system. It helps gather test execution data needed to troubleshoot failures. It manages both the collection and lookup of this data.

## Quick start

You can start i'Krelln by Docker:
 
 ```bash
docker run -d -p 7878:7878 ikrelln/ikrelln
```

Once it started, browse to http://localhost:7878 to find your tests results!

