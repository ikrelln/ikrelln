use std;
use actix::prelude::*;
use futures::future::*;
use std::collections::HashMap;

#[cfg(feature = "python")]
use cpython::{PyDict, Python, ToPyObject};

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

#[cfg(feature = "python")]
impl ToPyObject for Span {
    type ObjectType = PyDict;
    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        let object = PyDict::new(py);
        object
            .set_item(py, "trace_id", self.trace_id.clone())
            .unwrap();
        object.set_item(py, "id", self.id.clone()).unwrap();
        if let Some(parent_id) = self.parent_id.clone() {
            object.set_item(py, "parent_id", parent_id).unwrap();
        }
        if let Some(name) = self.name.clone() {
            object.set_item(py, "name", name).unwrap();
        }
        if let Some(kind) = self.kind.clone() {
            object.set_item(py, "kind", format!("{}", kind)).unwrap();
        }
        if let Some(duration) = self.duration.clone() {
            object.set_item(py, "duration", duration).unwrap();
        }
        if let Some(timestamp) = self.timestamp.clone() {
            object.set_item(py, "timestamp", timestamp).unwrap();
        }
        object.set_item(py, "tags", self.tags.clone()).unwrap();
        object
    }
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
    type Result = ();

    fn handle(
        &mut self,
        msg: super::ingestor::IngestEvents<Span>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        for event in &msg.events {
            let event = event.clone();
            Arbiter::handle().spawn(::DB_EXECUTOR_POOL.send(event.clone()).then(|span| {
                if let Ok(span) = span {
                    if let (Some(_), None) = (span.duration, span.parent_id.clone()) {
                        Arbiter::system_registry()
                            .get::<super::test::TraceParser>()
                            .do_send(super::test::TraceDoneNow(span.trace_id.clone()));
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
        println!("{:?}", span);
        assert!(span.is_ok());
    }
}
