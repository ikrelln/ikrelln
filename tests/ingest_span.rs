extern crate actix_web;
extern crate serde_json;
extern crate uuid;

extern crate ikrelln;

mod helpers;

use std::{thread, time};
use std::collections::HashMap;

use actix_web::*;

use ikrelln::engine::span::Span;
use ikrelln::api::span::IngestResponse;

#[test]
fn can_send_span() {
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();

    let req = srv.client(Method::POST, "/api/v1/spans")
        .json(vec![
            Span {
                trace_id: trace_id.to_string(),
                id: trace_id.clone(),
                parent_id: None,
                name: Some(trace_id.clone()),
                kind: Some(ikrelln::engine::span::Kind::CLIENT),
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
    assert_eq!(data.unwrap().nb_events, 1);

    thread::sleep(time::Duration::from_millis(100));

    let req_trace = srv.client(Method::GET, &format!("/api/v1/trace/{}", &trace_id))
        .finish()
        .unwrap();
    let response_trace = srv.execute(req_trace.send()).unwrap();
    println!("{:?}", response_trace);
    assert!(response_trace.status().is_success());
    let data_trace: Result<Vec<Span>, _> =
        serde_json::from_slice(&*srv.execute(response_trace.body()).unwrap());
    assert!(data_trace.is_ok());
    assert_eq!(data_trace.unwrap().len(), 1);
}

#[test]
fn can_send_spans() {
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();

    let req = srv.client(Method::POST, "/api/v1/spans")
        .json(vec![
            Span {
                trace_id: trace_id.to_string(),
                id: trace_id.clone(),
                parent_id: None,
                name: Some(trace_id.clone()),
                kind: Some(ikrelln::engine::span::Kind::CLIENT),
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
                kind: Some(ikrelln::engine::span::Kind::CLIENT),
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
                kind: Some(ikrelln::engine::span::Kind::CLIENT),
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

    thread::sleep(time::Duration::from_millis(100));

    let req_trace = srv.client(Method::GET, &format!("/api/v1/trace/{}", &trace_id))
        .finish()
        .unwrap();
    let response_trace = srv.execute(req_trace.send()).unwrap();
    println!("{:?}", response_trace);
    assert!(response_trace.status().is_success());
    let data_trace: Result<Vec<Span>, _> =
        serde_json::from_slice(&*srv.execute(response_trace.body()).unwrap());
    assert!(data_trace.is_ok());
    assert_eq!(data_trace.unwrap().len(), 3);
}
