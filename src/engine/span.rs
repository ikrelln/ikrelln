use actix::prelude::*;
use futures::future::*;

use opentracing::Span;

impl Handler<super::ingestor::IngestEvents<Span>> for super::ingestor::Ingestor {
    type Result = ();

    fn handle(
        &mut self,
        msg: super::ingestor::IngestEvents<Span>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        for event in &msg.events {
            let event = event.clone();
            Arbiter::spawn(::DB_EXECUTOR_POOL.send(event.clone()).then(|span| {
                if let Ok(span) = span {
                    if let (Some(_), None) = (span.duration, span.parent_id.clone()) {
                        actix::System::current()
                            .registry()
                            .get::<super::test_result::TraceParser>()
                            .do_send(super::test_result::TraceDone(span.trace_id.clone()));
                    }
                }
                result(Ok(()))
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use std;

    #[test]
    fn can_deserialize_zipkin_query() {
        let zipkin_query = r#"[
  {
    "traceId": "string",
    "name": "string",
    "parentId": "string",
    "id": "string",
    "kind": "CLIENT",
    "timestamp": 0,
    "duration": 0,
    "debug": true,
    "shared": true,
    "localEndpoint": {
      "serviceName": "string",
      "ipv4": "string",
      "ipv6": "string",
      "port": 0
    },
    "remoteEndpoint": {
      "serviceName": "string",
      "ipv4": "string",
      "ipv6": "string",
      "port": 0
    },
    "annotations": [
      {
        "timestamp": 0,
        "value": "string"
      }
    ],
    "tags": {
      "additionalProp1": "string",
      "additionalProp2": "string",
      "additionalProp3": "string"
    }
  }
]"#;

        let span: std::result::Result<Vec<super::Span>, _> = serde_json::from_str(zipkin_query);
        assert!(span.is_ok());
    }
}
