use std;
use serde;
use actix::*;


#[derive(Deserialize, Serialize, Debug, Clone)]
enum Status {
    SUCCESS,
    FAILURE,
    SKIPPED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestResult {
    test_name: String,
    result: Status,
    #[serde(deserialize_with = "deserialize_duration")] duration: std::time::Duration,
}

use serde::de::{self, Deserialize, MapAccess, Visitor};
fn deserialize_duration<'de, D>(
    deserializer: D,
) -> std::result::Result<std::time::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct IntOrStruct(std::marker::PhantomData<fn() -> std::time::Duration>);

    impl<'de> Visitor<'de> for IntOrStruct {
        type Value = std::time::Duration;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("int or map")
        }

        fn visit_u64<E>(self, value: u64) -> Result<std::time::Duration, E>
        where
            E: de::Error,
        {
            Ok(std::time::Duration::new(value, 0))
        }

        fn visit_map<M>(self, visitor: M) -> Result<std::time::Duration, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(IntOrStruct(std::marker::PhantomData))
}

#[derive(Debug)]
pub struct NewEvents {
    pub ingest_id: super::IngestId,
    events: Vec<TestResult>,
}
impl NewEvents {
    pub fn new(events: Vec<TestResult>) -> NewEvents {
        NewEvents {
            ingest_id: super::IngestId::new(),
            events: events,
        }
    }
}

impl ResponseType for NewEvents {
    type Item = ();
    type Error = ();
}

pub struct Ingestor;

impl Actor for Ingestor {
    type Context = Context<Self>;
}

impl Handler<NewEvents> for Ingestor {
    type Result = Result<(), ()>; // <- Message response type

    fn handle(&mut self, msg: NewEvents, _ctx: &mut Context<Self>) -> Self::Result {
        info!("{:?}", msg);
        Ok(())
    }
}
