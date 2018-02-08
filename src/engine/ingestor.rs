use actix::*;
use chrono;
use futures;

#[derive(Default)]
pub struct Ingestor;

impl Actor for Ingestor {
    type Context = Context<Self>;
}

impl Supervised for Ingestor {}
impl SystemService for Ingestor {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}


#[derive(Debug)]
pub struct IngestEvents<T> {
    pub ingest_id: super::IngestId,
    pub events: Vec<T>,
    pub created_at: chrono::NaiveDateTime,
    pub processed_at: Option<chrono::NaiveDateTime>,
}
impl<T> IngestEvents<T> {
    pub fn new(events: Vec<T>) -> IngestEvents<T> {
        IngestEvents {
            ingest_id: super::IngestId::new(),
            events: events,
            created_at: chrono::Utc::now().naive_utc(),
            processed_at: None,
        }
    }
    fn done(self) -> IngestEvents<T> {
        IngestEvents {
            processed_at: Some(chrono::Utc::now().naive_utc()),
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
            ::DB_EXECUTOR_POOL.send(::db::ingest_event::IngestEventDb::from(&fi.0.done()));
        }
        Ok(())
    }
}
