use actix_web::*;
use chrono;
use futures::future::result;
use futures::Future;
use std::collections::HashMap;

use super::{errors, AppState};

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub name: String,
    pub group: String,
    pub created_on: chrono::NaiveDateTime,
    pub last_update: chrono::NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<HashMap<String, Vec<crate::engine::test_result::TestResult>>>,
    pub environments: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<HashMap<crate::engine::test_result::TestStatus, usize>>,
}

pub fn get_reports(
    _req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    crate::DB_READ_EXECUTOR_POOL
        .send(crate::db::read::reports::GetAll)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

pub fn get_report(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match (
        req.match_info().get("reportGroup"),
        req.match_info().get("reportName"),
    ) {
        (Some(report_group), Some(report_name)) => crate::DB_READ_EXECUTOR_POOL
            .send(crate::db::read::reports::GetReport {
                report_group: report_group.to_string().replace("%20", " "),
                report_name: report_name.to_string().replace("%20", " "),
                environment: req.query().get("environment").map(|v| v.to_string()),
            })
            .from_err()
            .and_then(|res| match res {
                Some(report) => Ok(HttpResponse::Ok().json(report)),
                None => Err(super::errors::IkError::NotFound(
                    "report not found".to_string(),
                )),
            })
            .responder(),

        (_, _) => result(Err(super::errors::IkError::BadRequest(
            "missing path parameter".to_string(),
        )))
        .responder(),
    }
}
