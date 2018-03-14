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
use ikrelln::engine::test::TestResult;

#[test]
fn should_not_have_test_result_from_span_without_tags() {
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

    let req_tr = srv.client(
        Method::GET,
        &format!("/api/v1/testresult?traceId={}", &trace_id),
    ).finish()
        .unwrap();
    let response_tr = srv.execute(req_tr.send()).unwrap();
    println!("{:?}", response_tr);
    assert!(response_tr.status().is_client_error());
}

#[test]
fn should_create_test_result() {
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();

    let mut tags: HashMap<String, String> = HashMap::new();
    tags.insert(
        String::from({
            let tag: &str = ikrelln::engine::test::IkrellnTags::Suite.into();
            tag
        }),
        "test_suite".to_string(),
    );
    tags.insert(
        String::from({
            let tag: &str = ikrelln::engine::test::IkrellnTags::Class.into();
            tag
        }),
        "test_class".to_string(),
    );
    tags.insert(
        String::from({
            let tag: &str = ikrelln::engine::test::IkrellnTags::Result.into();
            tag
        }),
        "success".to_string(),
    );

    let req = srv.client(Method::POST, "/api/v1/spans")
        .json(vec![
            Span {
                trace_id: trace_id.to_string(),
                id: trace_id.clone(),
                parent_id: None,
                name: Some("span_name".to_string()),
                kind: Some(ikrelln::engine::span::Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: None,
                annotations: vec![],
                tags,
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

    let req_tr = srv.client(
        Method::GET,
        &format!("/api/v1/testresult?traceId={}", &trace_id),
    ).finish()
        .unwrap();
    let response_tr = srv.execute(req_tr.send()).unwrap();
    println!("{:?}", response_tr);
    assert!(response_tr.status().is_success());
    let data_tr: Result<Vec<TestResult>, _> =
        serde_json::from_slice(&*srv.execute(response_tr.body()).unwrap());
    assert!(data_tr.is_ok());
}
