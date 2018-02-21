use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
use futures::future::result;

use super::{errors, AppState};

#[derive(Serialize)]
pub struct TestItem {
    pub id: String,
    pub name: String,
}

pub fn get_tests_by_parent(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .call_fut(::db::test::GetTestItems(::db::test::TestItemQuery {
            parent_id: Some(req.query().get("parentId").map(|s| s.to_string())),
            ..Default::default()
        }))
        .from_err()
        .and_then(|res| match res {
            Ok(test_items) => Ok(httpcodes::HTTPOk.build().json(
                test_items
                    .iter()
                    .map(|item| TestItem {
                        id: item.test_id.clone(),
                        name: item.name.clone(),
                    })
                    .collect::<Vec<TestItem>>(),
            )?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}

pub fn get_test_results(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .call_fut(::db::test::GetTestResults(
            ::db::test::TestResultQuery::from_req(&req),
        ))
        .from_err()
        .and_then(|res| match res {
            Ok(test_results) => Ok(httpcodes::HTTPOk.build().json(test_results)?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}

#[derive(Serialize)]
pub struct TestDetails {
    pub test_id: String,
    pub name: String,
    pub path: Vec<TestItem>,
    pub children: Vec<TestItem>,
    pub last_traces: Vec<String>,
}
pub fn get_test(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("testId") {
        Some(test_id) => ::DB_EXECUTOR_POOL
            .call_fut(::db::test::GetTestItems(::db::test::TestItemQuery {
                id: Some(test_id.to_string()),
                with_children: true,
                with_full_path: true,
                with_traces: true,
                ..Default::default()
            }))
            .from_err()
            .and_then(|res| match res {
                Ok(test_results) => match test_results.get(0) {
                    Some(test_result) => Ok(httpcodes::HTTPOk.build().json(test_result)?),
                    None => Err(super::errors::IkError::NotFound(
                        "testId not found".to_string(),
                    )),
                },
                Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
            })
            .responder(),
        _ => result(Err(super::errors::IkError::BadRequest(
            "missing testId path parameter".to_string(),
        ))).responder(),
    }
}
