use std;
use serde;
use actix::*;
use futures::future::*;
use futures;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum Status {
    SUCCESS,
    FAILURE,
    SKIPPED,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub result: Status,
    #[serde(deserialize_with = "deserialize_duration")]
    pub duration: std::time::Duration,
    pub ts: Option<u64>,
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

impl Handler<::engine::ingestor::IngestEvents<TestResult>> for ::engine::ingestor::Ingestor {
    type Result = Result<(), ()>;

    fn handle(
        &mut self,
        msg: ::engine::ingestor::IngestEvents<TestResult>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        self.0
            .send(::db::ingest_event::StartIngestEventDb::from(&msg));
        let msg_futures = msg.events
            .iter()
            .map(|event: &TestResult| {
                self.0
                    .call_fut(::db::test_result::TestResultDb::from(event))
            })
            .collect::<Vec<_>>();
        let finishing = join_all(msg_futures)
            .and_then(|_| futures::future::result(Ok(::engine::ingestor::FinishedIngest(msg))));
        _ctx.add_future(finishing);
        Ok(())
    }
}
