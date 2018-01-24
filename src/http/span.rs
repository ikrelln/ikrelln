use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;

use super::{errors, AppState};
use engine::ingestor::IngestEvents;
use engine::span::Span;

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
        .and_then(move |val: Vec<Span>| {
            let nb_spans = val.len();
            let ingest = IngestEvents::new(val);
            let ingest_id = ingest.ingest_id.clone();
            debug!("ingesting {} event(s) as {}", nb_spans, ingest_id,);
            ingestor.borrow().send(ingest);
            Ok(httpcodes::HTTPOk.build().json(IngestResponse {
                ingest_id: ingest_id,
                nb_events: nb_spans,
            })?)
        })
        .responder()
}
