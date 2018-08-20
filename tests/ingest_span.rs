extern crate actix_web;
extern crate serde_json;
extern crate uuid;

extern crate ikrelln;

mod helpers;

use std::collections::HashMap;
use std::{thread, time};

use actix_web::*;

use ikrelln::api::span::IngestResponse;
use ikrelln::opentracing::span::Kind;
use ikrelln::opentracing::Span;

#[test]
fn can_receive_span() {
    helpers::setup_logger();
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();

    let req = srv
        .client(http::Method::POST, "/api/v1/spans")
        .json(vec![Span {
            trace_id: trace_id.to_string(),
            id: trace_id.clone(),
            parent_id: None,
            name: Some(trace_id.clone()),
            kind: Some(Kind::CLIENT),
            duration: Some(25),
            timestamp: Some(50),
            debug: false,
            shared: false,
            local_endpoint: None,
            remote_endpoint: None,
            annotations: vec![],
            tags: HashMap::new(),
            binary_annotations: vec![],
        }])
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());
    let data: Result<IngestResponse, _> =
        serde_json::from_slice(&*srv.execute(response.body()).unwrap());
    assert!(data.is_ok());
    assert_eq!(data.unwrap().nb_events, 1);

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_SPAN_SAVED_MILLISECONDS,
    ));

    let req_trace = srv
        .client(http::Method::GET, &format!("/api/v1/trace/{}", &trace_id))
        .finish()
        .unwrap();
    let response_trace = srv.execute(req_trace.send()).unwrap();
    assert!(response_trace.status().is_success());
    let data_trace: Result<Vec<Span>, _> =
        serde_json::from_slice(&*srv.execute(response_trace.body()).unwrap());
    assert!(data_trace.is_ok());
    println!("{:#?}", data_trace);
    assert_eq!(data_trace.unwrap().len(), 1);
    thread::sleep(time::Duration::from_millis(helpers::DELAY_FINISH));
}

#[test]
fn can_receive_spans() {
    helpers::setup_logger();
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();

    let req = srv
        .client(http::Method::POST, "/api/v1/spans")
        .json(vec![
            Span {
                trace_id: trace_id.to_string(),
                id: trace_id.clone(),
                parent_id: None,
                name: Some(trace_id.clone()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: None,
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(uuid::Uuid::new_v4().to_string()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: None,
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(uuid::Uuid::new_v4().to_string()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: None,
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
        ])
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());
    let data: Result<IngestResponse, _> =
        serde_json::from_slice(&*srv.execute(response.body()).unwrap());
    assert!(data.is_ok());
    assert_eq!(data.unwrap().nb_events, 3);

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_SPAN_SAVED_MILLISECONDS,
    ));

    let req_trace = srv
        .client(http::Method::GET, &format!("/api/v1/trace/{}", &trace_id))
        .finish()
        .unwrap();
    let response_trace = srv.execute(req_trace.send()).unwrap();
    assert!(response_trace.status().is_success());
    let data_trace: Result<Vec<Span>, _> =
        serde_json::from_slice(&*srv.execute(response_trace.body()).unwrap());
    assert!(data_trace.is_ok());
    assert_eq!(data_trace.unwrap().len(), 3);
    thread::sleep(time::Duration::from_millis(helpers::DELAY_FINISH));
}
