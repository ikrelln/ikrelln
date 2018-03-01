use actix::*;
use actix::registry::SystemService;
use chrono;

#[derive(Default)]
pub struct Ingestor;

impl Actor for Ingestor {
    type Context = Context<Self>;
}

impl Supervised for Ingestor {}
impl SystemService for Ingestor {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        info!("started Ingestor")
    }
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
            events,
            created_at: chrono::Utc::now().naive_utc(),
            processed_at: None,
        }
    }
}
impl<T> Message for IngestEvents<T> {
    type Result = ();
}
