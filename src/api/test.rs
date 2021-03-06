use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use futures::future::result;
use futures::Future;
use serde_urlencoded;

use super::{errors, AppState};

#[derive(Serialize)]
pub struct TestItem {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestResultsQueryParams {
    pub trace_id: Option<String>,
    pub status: Option<crate::engine::test_result::TestStatus>,
    pub test_id: Option<String>,
    pub environment: Option<String>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub ts: Option<i64>,
    pub lookback: Option<i64>,
    pub limit: Option<i64>,
}

pub fn get_test_results(
    req: &HttpRequest<AppState>,
) -> Box<dyn Future<Item = HttpResponse, Error = errors::IkError>> {
    match serde_urlencoded::from_str::<TestResultsQueryParams>(req.query_string()) {
        Ok(query_params) => crate::DB_READ_EXECUTOR_POOL
            .send(crate::db::read::test::GetTestResults(query_params.into()))
            .from_err()
            .and_then(|res| Ok(HttpResponse::Ok().json(res)))
            .responder(),
        Err(err) => result(Err(super::errors::IkError::BadRequest(format!(
            "invalid query parameters: '{}'",
            err
        ))))
        .responder(),
    }
}

#[derive(Serialize)]
pub struct TestDetails {
    pub test_id: String,
    pub name: String,
    pub path: Vec<TestItem>,
    pub children: Vec<TestItem>,
    pub last_results: Vec<crate::engine::test_result::TestResult>,
}
pub fn get_test(
    req: &HttpRequest<AppState>,
) -> Box<dyn Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("testId") {
        Some(test_id) => crate::DB_READ_EXECUTOR_POOL
            .send(match test_id {
                "root" => {
                    crate::db::read::test::GetTestItems(crate::db::read::test::TestItemQuery {
                        id: None,
                        parent_id: Some("root".to_string()),
                        with_children: true,
                        with_full_path: true,
                        with_traces: true,
                    })
                }
                _ => crate::db::read::test::GetTestItems(crate::db::read::test::TestItemQuery {
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
                1 => Ok(HttpResponse::Ok().json(res.get(0))),
                _ => Ok(HttpResponse::Ok().json(TestDetails {
                    test_id: "root".to_string(),
                    name: "".to_string(),
                    path: vec![],
                    children: res
                        .iter()
                        .map(|tr| TestItem {
                            name: tr.name.clone(),
                            id: tr.test_id.clone(),
                        })
                        .collect(),
                    last_results: vec![],
                })),
            })
            .responder(),
        _ => result(Err(super::errors::IkError::BadRequest(
            "missing testId path parameter".to_string(),
        )))
        .responder(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetTestQueryParams {
    parent_id: String,
}

pub fn get_tests_by_parent(
    req: &HttpRequest<AppState>,
) -> Box<dyn Future<Item = HttpResponse, Error = errors::IkError>> {
    match serde_urlencoded::from_str::<GetTestQueryParams>(req.query_string()) {
        Ok(params) => crate::DB_READ_EXECUTOR_POOL
            .send(crate::db::read::test::GetTestItems(
                crate::db::read::test::TestItemQuery {
                    parent_id: Some(params.parent_id),
                    with_children: true,
                    with_full_path: true,
                    with_traces: true,
                    ..Default::default()
                },
            ))
            .from_err()
            .and_then(|res| match res.len() {
                0 => Err(super::errors::IkError::NotFound(
                    "testId not found".to_string(),
                )),
                _ => Ok(HttpResponse::Ok().json(res)),
            })
            .responder(),
        _ => result(Err(super::errors::IkError::BadRequest(
            "missing parentId query parameter".to_string(),
        )))
        .responder(),
    }
}

pub fn get_environments(
    _req: &HttpRequest<AppState>,
) -> Box<dyn Future<Item = HttpResponse, Error = errors::IkError>> {
    crate::DB_READ_EXECUTOR_POOL
        .send(crate::db::read::test::GetEnvironments)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}
