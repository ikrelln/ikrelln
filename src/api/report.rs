use std::collections::HashMap;
use actix_web::*;
//use actix;
use futures::Future;
use futures::future::result;
use chrono;

use super::{errors, AppState};

#[derive(Serialize, Debug)]
pub struct Report {
    pub name: String,
    pub created_on: chrono::NaiveDateTime,
    pub last_update: chrono::NaiveDateTime,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categories: Option<HashMap<String, Vec<::engine::test::TestResult>>>,
    pub environments: Vec<String>,
}

pub fn get_reports(
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .send(::db::reports::GetAll)
        .from_err()
        .and_then(|res| Ok(httpcodes::HTTPOk.build().json(res)?))
        .responder()
}

pub fn get_report(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("reportName") {
        Some(report_name) => ::DB_EXECUTOR_POOL
            .send(::db::reports::GetReport {
                report_name: report_name.to_string().replace("%20", " "),
                environment: req.query().get("environment").map(|v| v.to_string()),
            })
            .from_err()
            .and_then(|res| match res {
                Some(report) => Ok(httpcodes::HTTPOk.build().json(report)?),
                None => Err(super::errors::IkError::NotFound(
                    "report not found".to_string(),
                )),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing reportName path parameter".to_string(),
        ))).responder(),
    }
}
