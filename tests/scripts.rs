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
use ikrelln::api::span::IngestResponse;
use ikrelln::opentracing::span::Endpoint;
use ikrelln::api::report::Report;
use ikrelln::engine::streams::{Script, ScriptType};

#[test]
fn can_save_report_script() {
    helpers::setup_logger();
    let mut srv = helpers::setup_server();

    let script_name = uuid::Uuid::new_v4().to_string();

    let req = srv.client(Method::POST, "/api/v1/scripts")
        .json(Script {
            name: script_name.clone(),
            script_type: ScriptType::ReportFilterTestResult,
            source: "def reports_for_test(test):\n  import json\n  report = test['main_span']['tags']['report']\n  return [{'group': 'from_script', 'name': report, 'category': test['path'][1]}]".to_string(),
            ..Default::default()
        })
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());
    let data: Result<Script, _> = serde_json::from_slice(&*srv.execute(response.body()).unwrap());
    assert!(data.is_ok());
    let script_sent = data.unwrap();
    assert!(&script_sent.id.is_some());
    assert_eq!(script_sent.name.clone(), script_name.clone());

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_SCRIPT_SAVED_MILLISECONDS,
    ));

    let req_script = srv.client(
        Method::GET,
        &format!("/api/v1/scripts/{}", &script_sent.id.unwrap()),
    ).finish()
        .unwrap();
    let response_script = srv.execute(req_script.send()).unwrap();
    assert!(response_script.status().is_success());
    let data_script: Result<Script, _> =
        serde_json::from_slice(&*srv.execute(response_script.body()).unwrap());
    assert!(data_script.is_ok());
    assert_eq!(data_script.unwrap().name, script_name.clone());
    thread::sleep(time::Duration::from_millis(helpers::DELAY_FINISH));
}

#[test]
fn can_create_report_from_script() {
    helpers::setup_logger();
    let mut srv = helpers::setup_server();

    let script_name = uuid::Uuid::new_v4().to_string();

    let req = srv.client(Method::POST, "/api/v1/scripts")
        .json(Script {
            name: script_name.clone(),
            script_type: ScriptType::ReportFilterTestResult,
            source: "def reports_for_test(test):\n  import json\n  report = test['main_span']['tags']['report']\n  return [{'group': 'from_script', 'name': report, 'category': test['path'][1]}]".to_string(),
            ..Default::default()
        })
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());
    let data: Result<Script, _> = serde_json::from_slice(&*srv.execute(response.body()).unwrap());
    assert!(data.is_ok());
    let script_sent = data.unwrap();
    assert!(&script_sent.id.is_some());
    assert_eq!(script_sent.name.clone(), script_name.clone());

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_SCRIPT_SAVED_MILLISECONDS,
    ));

    let trace_id = uuid::Uuid::new_v4().to_string();
    let service_name = uuid::Uuid::new_v4().to_string();
    let test_name = uuid::Uuid::new_v4().to_string();

    let report_name = uuid::Uuid::new_v4().to_string();

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
    tags.insert("report".to_string(), report_name.clone());

    let spans = vec![
        Span {
            trace_id: trace_id.to_string(),
            id: trace_id.clone(),
            parent_id: None,
            name: Some(test_name.clone()),
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
            name: Some("sub span".to_string()),
            kind: Some(Kind::CLIENT),
            duration: Some(25),
            timestamp: Some(50),
            debug: false,
            shared: false,
            local_endpoint: None,
            remote_endpoint: Some(Endpoint {
                service_name: Some(service_name.clone()),
                ..Default::default()
            }),
            annotations: vec![],
            tags: HashMap::new(),
            binary_annotations: vec![],
        },
    ];
    println!("{:?}", spans);
    let req = srv.client(Method::POST, "/api/v1/spans")
        .json(spans)
        .unwrap();
    let response = srv.execute(req.send()).unwrap();
    assert!(response.status().is_success());
    let data: Result<IngestResponse, _> =
        serde_json::from_slice(&*srv.execute(response.body()).unwrap());
    assert!(data.is_ok());
    assert_eq!(data.unwrap().nb_events, 2);

    thread::sleep(time::Duration::from_millis(
        helpers::DELAY_REPORT_SAVED_MILLISECONDS,
    ));

    let req_report = srv.client(
        Method::GET,
        &format!("/api/v1/reports/from_script/{}", report_name.clone()),
    ).finish()
        .unwrap();
    let response_report = srv.execute(req_report.send()).unwrap();
    assert!(response_report.status().is_success());
    let data_report_res: Result<Report, _> =
        serde_json::from_slice(&*srv.execute(response_report.body()).unwrap());
    assert!(data_report_res.is_ok());
    let data_report = data_report_res.unwrap();
    assert_eq!(data_report.group, "from_script".to_string());
    assert_eq!(data_report.name, report_name.clone());
    let categories = data_report.categories.unwrap();
    assert!(categories.contains_key("test_class"));
    assert!(
        categories
            .get("test_class")
            .unwrap()
            .iter()
            .map(|tr| tr.name.clone())
            .collect::<Vec<String>>()
            .contains(&test_name)
    );
    thread::sleep(time::Duration::from_millis(helpers::DELAY_FINISH));
}
