use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
use futures::future::result;
use std::collections::HashMap;

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
            ingestor.send(ingest);
            Ok(httpcodes::HTTPOk.build().json(IngestResponse {
                ingest_id: ingest_id,
                nb_events: nb_spans,
            })?)
        })
        .responder()
}


pub fn get_services(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.state()
        .db_actor
        .call_fut(::db::span::GetServices)
        .from_err()
        .and_then(|res| match res {
            Ok(services) => Ok(httpcodes::HTTPOk.build().json(services)?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}

pub fn get_spans_by_service(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.query().get("serviceName") {
        Some(_) => req.state()
            .db_actor
            .call_fut(::db::span::GetSpans(::db::span::SpanQuery::from_req(&req)))
            .from_err()
            .and_then(|res| match res {
                Ok(spans) => Ok(httpcodes::HTTPOk.build().json(spans)?),
                Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing serviceName query parameter".to_string(),
        ))).responder(),
    }
}

pub fn get_spans_by_trace_id(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("traceId") {
        Some(trace_id) => req.state()
            .db_actor
            .call_fut(::db::span::GetSpans(
                ::db::span::SpanQuery::from_req(&req).with_trace_id(trace_id.to_string()),
            ))
            .from_err()
            .and_then(|res| match res {
                Ok(spans) => Ok(httpcodes::HTTPOk.build().json(spans)?),
                Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing traceId path parameter".to_string(),
        ))).responder(),
    }
}

pub fn get_traces(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.state()
        .db_actor
        .call_fut(::db::span::GetSpans(::db::span::SpanQuery::from_req(&req)))
        .from_err()
        .and_then(|res| match res {
            Ok(spans) => Ok(httpcodes::HTTPOk.build().json({
                let mut by_trace_with_key = HashMap::new();
                for span in spans.into_iter() {
                    by_trace_with_key
                        .entry(span.trace_id.clone())
                        .or_insert(Vec::new())
                        .push(span);
                }
                let mut by_trace = Vec::new();
                for (_, spans) in by_trace_with_key.into_iter() {
                    by_trace.push(spans);
                }
                by_trace
            })?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}
