use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
use futures::future::result;

use super::{errors, AppState};

#[derive(Serialize)]
pub struct TestItem {
    pub id: String,
    pub name: String,
}

pub fn get_test_results(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .send(::db::test::GetTestResults(
            ::db::test::TestResultQuery::from_req(&req),
        ))
        .from_err()
        .and_then(|res| Ok(httpcodes::HTTPOk.build().json(res)?))
        .responder()
}

#[derive(Serialize)]
pub struct TestDetails {
    pub test_id: String,
    pub name: String,
    pub path: Vec<TestItem>,
    pub children: Vec<TestItem>,
    pub last_results: Vec<::engine::test::TestResult>,
}
pub fn get_test(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("testId") {
        Some(test_id) => ::DB_EXECUTOR_POOL
            .send(match test_id {
                "root" => ::db::test::GetTestItems(::db::test::TestItemQuery {
                    id: None,
                    parent_id: Some("root".to_string()),
                    with_children: true,
                    with_full_path: true,
                    with_traces: true,
                }),
                _ => ::db::test::GetTestItems(::db::test::TestItemQuery {
                    id: Some(test_id.to_string()),
                    with_children: true,
                    with_full_path: true,
                    with_traces: true,
                    ..Default::default()
                }),
            })
            .from_err()
            .and_then(|res| match res.len() {
                0 => Err(super::errors::IkError::NotFound(
                    "testId not found".to_string(),
                )),
                1 => Ok(httpcodes::HTTPOk.build().json(res.get(0))?),
                _ => Ok(httpcodes::HTTPOk.build().json(TestDetails {
                    test_id: "root".to_string(),
                    name: "".to_string(),
                    path: vec![],
                    children: res.iter()
                        .map(|tr| TestItem {
                            name: tr.name.clone(),
                            id: tr.test_id.clone(),
                        })
                        .collect(),
                    last_results: vec![],
                })?),
            })
            .responder(),
        _ => result(Err(super::errors::IkError::BadRequest(
            "missing testId path parameter".to_string(),
        ))).responder(),
    }
}

pub fn get_tests_by_parent(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.query().get("parentId") {
        Some(test_id) => ::DB_EXECUTOR_POOL
            .send(::db::test::GetTestItems(::db::test::TestItemQuery {
                parent_id: Some(test_id.to_string()),
                with_children: true,
                with_full_path: true,
                with_traces: true,
                ..Default::default()
            }))
            .from_err()
            .and_then(|res| match res.len() {
                0 => Err(super::errors::IkError::NotFound(
                    "testId not found".to_string(),
                )),
                _ => Ok(httpcodes::HTTPOk.build().json(res)?),
            })
            .responder(),
        _ => result(Err(super::errors::IkError::BadRequest(
            "missing parentId query parameter".to_string(),
        ))).responder(),
    }
}

pub fn get_environments(
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .send(::db::test::GetEnvironments)
        .from_err()
        .and_then(|res| Ok(httpcodes::HTTPOk.build().json(res)?))
        .responder()
}
