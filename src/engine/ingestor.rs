use std;
use serde;
use actix::*;
use chrono;
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
    #[serde(deserialize_with = "deserialize_duration")] pub duration: std::time::Duration,
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

#[derive(Debug)]
pub struct IngestEvents {
    pub ingest_id: super::IngestId,
    events: Vec<TestResult>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl IngestEvents {
    pub fn new(events: Vec<TestResult>) -> IngestEvents {
        IngestEvents {
            ingest_id: super::IngestId::new(),
            events: events,
            created_at: chrono::Utc::now(),
            processed_at: None,
        }
    }
    fn done(self) -> IngestEvents {
        IngestEvents {
            processed_at: Some(chrono::Utc::now()),
            ..self
        }
    }
}

impl ResponseType for IngestEvents {
    type Item = ();
    type Error = ();
}
pub struct Ingestor(pub SyncAddress<::db::DbExecutor>);

impl Actor for Ingestor {
    type Context = Context<Self>;
}

impl Handler<IngestEvents> for Ingestor {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: IngestEvents, _ctx: &mut Context<Self>) -> Self::Result {
        self.0
            .send(::db::ingest_event::StartIngestEventDb::from(&msg));
        let msg_futures = msg.events
            .iter()
            .map(|event| {
                self.0
                    .call_fut(::db::test_result::TestResultDb::from(event))
            })
            .collect::<Vec<_>>();
        let finishing =
            join_all(msg_futures).and_then(|_| futures::future::result(Ok(FinishedIngest(msg))));
        _ctx.add_future(finishing);
        Ok(())
    }
}

#[derive(Debug)]
struct FinishedIngest(IngestEvents);

impl ResponseType for FinishedIngest {
    type Item = ();
    type Error = ();
}

impl Handler<Result<FinishedIngest, futures::Canceled>> for Ingestor {
    type Result = Result<(), ()>;
    fn handle(
        &mut self,
        msg: Result<FinishedIngest, futures::Canceled>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        if let Ok(fi) = msg {
            self.0
                .send(::db::ingest_event::FinishIngestEventDb::from(&fi.0.done()));
        }
        Ok(())
    }
}
