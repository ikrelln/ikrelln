use actix::*;
use chrono;
use futures;

pub struct Ingestor(pub SyncAddress<::db::DbExecutor>);

impl Actor for Ingestor {
    type Context = Context<Self>;
}


#[derive(Debug)]
pub struct IngestEvents<T> {
    pub ingest_id: super::IngestId,
    pub events: Vec<T>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
}
impl<T> IngestEvents<T> {
    pub fn new(events: Vec<T>) -> IngestEvents<T> {
        IngestEvents {
            ingest_id: super::IngestId::new(),
            events: events,
            created_at: chrono::Utc::now(),
            processed_at: None,
        }
    }
    fn done(self) -> IngestEvents<T> {
        IngestEvents {
            processed_at: Some(chrono::Utc::now()),
            ..self
        }
    }
}
impl<T> ResponseType for IngestEvents<T> {
    type Item = ();
    type Error = ();
}

#[derive(Debug)]
pub struct FinishedIngest<T>(pub IngestEvents<T>);
impl<T> ResponseType for FinishedIngest<T> {
    type Item = ();
    type Error = ();
}
impl<T> Handler<Result<FinishedIngest<T>, futures::Canceled>> for Ingestor {
    type Result = Result<(), ()>;
    fn handle(
        &mut self,
        msg: Result<FinishedIngest<T>, futures::Canceled>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        if let Ok(fi) = msg {
            self.0
                .send(::db::ingest_event::FinishIngestEventDb::from(&fi.0.done()));
        }
        Ok(())
    }
}
