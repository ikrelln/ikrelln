use actix_web::*;
use actix;
use futures::Future;
use futures::future::result;
use uuid;
use chrono;

use super::{errors, AppState};

pub fn save_script(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.json()
        .from_err()
        .and_then(move |script: ::engine::streams::Script| {
            let new_script = ::engine::streams::Script {
                id: match script.script_type {
                    ::engine::streams::ScriptType::UITest => {
                        Some(::engine::streams::ScriptType::UITest.into())
                    }
                    ::engine::streams::ScriptType::UITestResult => {
                        Some(::engine::streams::ScriptType::UITestResult.into())
                    }
                    _ => Some(uuid::Uuid::new_v4().hyphenated().to_string()),
                },
                status: Some(::engine::streams::ScriptStatus::Enabled),
                date_added: Some(chrono::Utc::now().naive_utc()),
                ..script
            };
            ::DB_EXECUTOR_POOL.do_send(::db::scripts::SaveScript(new_script.clone()));
            match new_script.script_type {
                ::engine::streams::ScriptType::StreamTest
                | ::engine::streams::ScriptType::ReportFilterTestResult => {
                    actix::Arbiter::system_registry()
                        .get::<::engine::streams::Streamer>()
                        .do_send(::engine::streams::AddScript(new_script.clone()))
                }
                _ => (),
            }
            Ok(httpcodes::HTTPOk.build().json(new_script)?)
        })
        .responder()
}

pub fn get_script(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("scriptId") {
        Some(script_id) => ::DB_EXECUTOR_POOL
            .send(::db::scripts::GetScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Some(script) => Ok(httpcodes::HTTPOk.build().json(script)?),
                None => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing scriptId path parameter".to_string(),
        ))).responder(),
    }
}

pub fn delete_script(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("scriptId") {
        Some(script_id) => ::DB_EXECUTOR_POOL
            .send(::db::scripts::DeleteScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Some(script) => {
                    match script.script_type {
                        ::engine::streams::ScriptType::StreamTest
                        | ::engine::streams::ScriptType::ReportFilterTestResult => {
                            actix::Arbiter::system_registry()
                                .get::<::engine::streams::Streamer>()
                                .do_send(::engine::streams::RemoveScript(script.clone()))
                        }
                        _ => (),
                    }

                    Ok(httpcodes::HTTPOk.build().json(script)?)
                }
                None => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
            })
            .responder(),

        _ => result(Err(super::errors::IkError::BadRequest(
            "missing scriptId path parameter".to_string(),
        ))).responder(),
    }
}

pub fn list_scripts(
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .send(::db::scripts::GetAll(None))
        .from_err()
        .and_then(|res| Ok(httpcodes::HTTPOk.build().json(res)?))
        .responder()
}

pub fn update_script(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.clone()
        .json()
        .from_err()
        .and_then(
            move |script: ::engine::streams::Script| match req.match_info().get("scriptId") {
                Some(script_id) => {
                    let new_script = ::engine::streams::Script {
                        id: Some(script_id.to_string()),
                        ..script
                    };
                    ::DB_EXECUTOR_POOL.do_send(::db::scripts::UpdateScript(new_script.clone()));
                    match new_script.script_type {
                        ::engine::streams::ScriptType::StreamTest
                        | ::engine::streams::ScriptType::ReportFilterTestResult => {
                            actix::Arbiter::system_registry()
                                .get::<::engine::streams::Streamer>()
                                .do_send(::engine::streams::UpdateScript(new_script.clone()))
                        }
                        _ => (),
                    }
                    Ok(httpcodes::HTTPOk.build().json(new_script)?)
                }
                _ => Err(super::errors::IkError::BadRequest(
                    "missing scriptId path parameter".to_string(),
                )),
            },
        )
        .responder()
}
