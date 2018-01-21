use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;

use super::{errors, AppState};
use engine::ingestor::{NewEvents, TestResult};

#[derive(Debug, Serialize)]
struct IngestResponse {
    ingest_id: ::engine::IngestId,
    nb_events: usize,
}

pub fn ingest(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    let ingestor = req.state().ingestor.clone();
    req.json()
        .from_err()
        .and_then(move |val: Vec<TestResult>| {
            let ingest = NewEvents::new(val.iter().cloned().collect());
            let ingest_id = ingest.ingest_id.clone();
            debug!(
                "ingesting {} event(s) as {}: {:?}",
                val.len(),
                ingest_id,
                val
            );
            ingestor.borrow().send(ingest);
            Ok(httpcodes::HTTPOk.build().json(IngestResponse {
                ingest_id: ingest_id,
                nb_events: val.len(),
            })?)
        })
        .responder()
}
