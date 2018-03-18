extern crate actix_web;
extern crate serde_json;
extern crate uuid;

extern crate ikrelln;

mod helpers;

use std::{thread, time};
use std::collections::HashMap;

use actix_web::*;

use ikrelln::opentracing::Span;
use ikrelln::opentracing::span::Kind;
use ikrelln::opentracing::tags::IkrellnTags;
use ikrelln::opentracing::span::Endpoint;
use ikrelln::api::span::IngestResponse;
use ikrelln::api::report::Report;

#[test]
fn should_create_report() {
    helpers::setup_logger();
    let mut srv = helpers::setup_server();

    let trace_id = uuid::Uuid::new_v4().to_string();
    let service_name1 = uuid::Uuid::new_v4().to_string();
    let service_name2 = uuid::Uuid::new_v4().to_string();
    let reported_span1 = uuid::Uuid::new_v4().to_string();
    let reported_span2 = uuid::Uuid::new_v4().to_string();

    let mut tags: HashMap<String, String> = HashMap::new();
    tags.insert(
        String::from({
            let tag: &str = IkrellnTags::Suite.into();
            tag
        }),
        "test_suite".to_string(),
    );
    tags.insert(
        String::from({
            let tag: &str = IkrellnTags::Class.into();
            tag
        }),
        "test_class".to_string(),
    );
    tags.insert(
        String::from({
            let tag: &str = IkrellnTags::Result.into();
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
                kind: Some(Kind::CLIENT),
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
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(reported_span1.clone()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: Some(Endpoint {
                    service_name: Some(service_name1.clone()),
                    ..Default::default()
                }),
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(reported_span1.clone()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: Some(Endpoint {
                    service_name: Some(service_name1.clone()),
                    ..Default::default()
                }),
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(reported_span1.clone()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: Some(Endpoint {
                    service_name: Some(service_name2.clone()),
                    ..Default::default()
                }),
                annotations: vec![],
                tags: HashMap::new(),
                binary_annotations: vec![],
            },
            Span {
                trace_id: trace_id.clone(),
                id: uuid::Uuid::new_v4().to_string(),
                parent_id: Some(trace_id.clone()),
                name: Some(reported_span2.clone()),
                kind: Some(Kind::CLIENT),
                duration: Some(25),
                timestamp: Some(50),
                debug: false,
                shared: false,
                local_endpoint: None,
                remote_endpoint: Some(Endpoint {
                    service_name: Some(service_name2.clone()),
                    ..Default::default()
                }),
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
    assert_eq!(data.unwrap().nb_events, 5);

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_REPORT_SAVED_MILLISECONDS,
    ));

    {
        let req_report = srv.client(
            Method::GET,
            &format!("/api/v1/reports/endpoints/{}", service_name1),
        ).finish()
            .unwrap();
        let response_report = srv.execute(req_report.send()).unwrap();
        assert!(response_report.status().is_success());
        let data_report_res: Result<Report, _> =
            serde_json::from_slice(&*srv.execute(response_report.body()).unwrap());
        assert!(data_report_res.is_ok());
        let data_report = data_report_res.unwrap();
        assert_eq!(data_report.group, "endpoints".to_string());
        assert_eq!(data_report.name, service_name1);
        assert!(
            data_report
                .categories
                .unwrap()
                .contains_key(&reported_span1)
        );
    }
    {
        let req_report = srv.client(
            Method::GET,
            &format!("/api/v1/reports/endpoints/{}", service_name2),
        ).finish()
            .unwrap();
        let response_report = srv.execute(req_report.send()).unwrap();
        assert!(response_report.status().is_success());
        let data_report_res: Result<Report, _> =
            serde_json::from_slice(&*srv.execute(response_report.body()).unwrap());
        assert!(data_report_res.is_ok());
        let data_report = data_report_res.unwrap();
        assert_eq!(data_report.group, "endpoints".to_string());
        assert_eq!(data_report.name, service_name2);
        let categories = data_report.categories.unwrap();
        assert!(categories.contains_key(&reported_span1));
        assert!(categories.contains_key(&reported_span2));
    }
    thread::sleep(time::Duration::from_millis(helpers::DELAY_FINISH));
}
