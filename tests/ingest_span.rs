extern crate actix_web;
extern crate serde_json;

extern crate ikrelln;

use std::collections::HashMap;

use actix_web::test::TestServer;
use actix_web::*;

use ikrelln::api::http_application;
use ikrelln::engine::span::Span;
use ikrelln::api::span::IngestResponse;

#[test]
fn can_send_span() {
    let mut srv = TestServer::with_factory(http_application);

    let req = srv.client(Method::POST, "/api/v1/spans")
        .json(vec![
            Span {
                trace_id: "1234".to_string(),
                id: "1234".to_string(),
                parent_id: None,
                name: Some("1234".to_string()),
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
}
