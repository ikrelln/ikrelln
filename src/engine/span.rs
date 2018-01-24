use actix::*;
use futures::future::*;
use futures;

use db::schema::span;
#[derive(Debug, Deserialize, Insertable, Clone)]
#[table_name = "span"]
#[serde(rename_all = "camelCase")]
pub struct Span {
    trace_id: String,
    parent_id: Option<String>,
    id: String,
    name: Option<String>,
    duration: i64,
    #[column_name = "ts"]
    timestamp: i64,
}

impl Handler<super::ingestor::IngestEvents<Span>> for super::ingestor::Ingestor {
    type Result = Result<(), ()>;

    fn handle(
        &mut self,
        msg: super::ingestor::IngestEvents<Span>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.0
            .send(::db::ingest_event::StartIngestEventDb::from(&msg));
        let msg_futures = msg.events
            .iter()
            .map(move |event: &Span| self.0.call_fut(event.clone()))
            .collect::<Vec<_>>();
        let finishing = join_all(msg_futures)
            .and_then(|_| futures::future::result(Ok(super::ingestor::FinishedIngest(msg))));
        _ctx.add_future(finishing);
        Ok(())
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
