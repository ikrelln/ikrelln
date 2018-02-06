use std;
use actix::*;
use futures::future::*;
use futures;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Kind {
    CLIENT,
    SERVER,
    PRODUCER,
    CONSUMER,
}
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<String> for Kind {
    fn from(string: String) -> Self {
        match string.as_str() {
            "CLIENT" => Kind::CLIENT,
            "SERVER" => Kind::SERVER,
            "PRODUCER" => Kind::PRODUCER,
            "CONSUMER" => Kind::CONSUMER,
            _ => Kind::CLIENT,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub trace_id: String,
    pub id: String,
    pub parent_id: Option<String>,
    pub name: Option<String>,
    pub kind: Option<Kind>,
    pub duration: Option<i64>,
    pub timestamp: Option<i64>,
    #[serde(default)] pub debug: bool,
    #[serde(default)] pub shared: bool,
    pub local_endpoint: Option<Endpoint>,
    pub remote_endpoint: Option<Endpoint>,
    #[serde(default)] pub annotations: Vec<Annotation>,
    #[serde(default)] pub tags: HashMap<String, String>,
    #[serde(default)] pub binary_annotations: Vec<BinaryTag>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    pub service_name: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub port: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Annotation {
    pub value: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BinaryTag {
    pub key: String,
    pub value: String,
    pub endpoint: Option<Endpoint>,
}

impl Handler<super::ingestor::IngestEvents<Span>> for super::ingestor::Ingestor {
    type Result = Result<(), ()>;

    fn handle(
        &mut self,
        msg: super::ingestor::IngestEvents<Span>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.0.send(::db::ingest_event::IngestEventDb::from(&msg));
        let msg_futures = msg.events
            .iter()
            .map(move |event: &Span| {
                match (event.duration, event.parent_id.clone()) {
                    (Some(_), None) => Arbiter::registry()
                        .get::<super::batcher::Batcher>()
                        .send(super::batcher::Register(event.trace_id.clone())),
                    _ => (),
                }
                self.0.call_fut(event.clone())
            })
            .collect::<Vec<_>>();
        let finishing = join_all(msg_futures).and_then(|_| {
            futures::future::result(Ok(super::ingestor::FinishedIngest(msg)))
        });
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
        println!("{:?}", span);
        assert!(span.is_ok());
    }
}
