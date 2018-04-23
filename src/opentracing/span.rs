use std::fmt::{Display, Formatter, Result};
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
impl Display for Kind {
    fn fmt(&self, f: &mut Formatter) -> Result {
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
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub shared: bool,
    pub local_endpoint: Option<Endpoint>,
    pub remote_endpoint: Option<Endpoint>,
    #[serde(default)]
    pub annotations: Vec<Annotation>,
    #[serde(default)]
    pub tags: HashMap<String, String>,
    #[serde(default)]
    pub binary_annotations: Vec<BinaryTag>,
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

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
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
