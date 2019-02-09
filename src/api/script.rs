use actix;
use actix_web::{AsyncResponder, HttpMessage, HttpRequest, HttpResponse};
use chrono;
use futures::future::result;
use futures::Future;
use uuid;

use super::{errors, AppState};

pub fn save_script(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.json()
        .from_err()
        .and_then(move |script: crate::engine::streams::Script| {
            let new_script = crate::engine::streams::Script {
                id: match script.script_type {
                    crate::engine::streams::ScriptType::UITest => {
                        Some(crate::engine::streams::ScriptType::UITest.into())
                    }
                    crate::engine::streams::ScriptType::UITestResult => {
                        Some(crate::engine::streams::ScriptType::UITestResult.into())
                    }
                    _ => Some(uuid::Uuid::new_v4().to_hyphenated().to_string()),
                },
                status: Some(crate::engine::streams::ScriptStatus::Enabled),
                date_added: Some(chrono::Utc::now().naive_utc()),
                ..script
            };
            crate::DB_EXECUTOR_POOL.do_send(crate::db::scripts::SaveScript(new_script.clone()));
            match new_script.script_type {
                crate::engine::streams::ScriptType::StreamTest
                | crate::engine::streams::ScriptType::ReportFilterTestResult => {
                    actix::System::current()
                        .registry()
                        .get::<crate::engine::streams::Streamer>()
                        .do_send(crate::engine::streams::AddScript(new_script.clone()))
                }
                _ => (),
            }
            Ok(HttpResponse::Ok().json(new_script))
        })
        .responder()
}

pub fn get_script(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("scriptId") {
        Some(script_id) => crate::DB_READ_EXECUTOR_POOL
            .send(crate::db::read::scripts::GetScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Some(script) => Ok(HttpResponse::Ok().json(script)),
                None => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing scriptId path parameter".to_string(),
        )))
        .responder(),
    }
}

pub fn delete_script(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("scriptId") {
        Some(script_id) => crate::DB_EXECUTOR_POOL
            .send(crate::db::scripts::DeleteScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Some(script) => {
                    match script.script_type {
                        crate::engine::streams::ScriptType::StreamTest
                        | crate::engine::streams::ScriptType::ReportFilterTestResult => {
                            actix::System::current()
                                .registry()
                                .get::<crate::engine::streams::Streamer>()
                                .do_send(crate::engine::streams::RemoveScript(script.clone()))
                        }
                        _ => (),
                    }

                    Ok(HttpResponse::Ok().json(script))
                }
                None => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing scriptId path parameter".to_string(),
        )))
        .responder(),
    }
}

pub fn list_scripts(
    _req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    crate::DB_READ_EXECUTOR_POOL
        .send(crate::db::read::scripts::GetAll(None))
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

pub fn update_script(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    let match_info = req.match_info().clone();
    req.json()
        .from_err()
        .and_then(
            move |script: crate::engine::streams::Script| match match_info.get("scriptId") {
                Some(script_id) => {
                    let new_script = crate::engine::streams::Script {
                        id: Some(script_id.to_string()),
                        ..script
                    };
                    crate::DB_EXECUTOR_POOL
                        .do_send(crate::db::scripts::UpdateScript(new_script.clone()));
                    match new_script.script_type {
                        crate::engine::streams::ScriptType::StreamTest
                        | crate::engine::streams::ScriptType::ReportFilterTestResult => {
                            actix::System::current()
                                .registry()
                                .get::<crate::engine::streams::Streamer>()
                                .do_send(crate::engine::streams::UpdateScript(new_script.clone()))
                        }
                        _ => (),
                    }
                    Ok(HttpResponse::Ok().json(new_script))
                }
                _ => Err(super::errors::IkError::BadRequest(
                    "missing scriptId path parameter".to_string(),
                )),
            },
        )
        .responder()
}

pub fn reload_scripts(_req: &HttpRequest<AppState>) -> HttpResponse {
    actix::System::current()
        .registry()
        .get::<crate::engine::streams::Streamer>()
        .do_send(crate::engine::streams::LoadScripts);
    HttpResponse::Ok().finish()
}
