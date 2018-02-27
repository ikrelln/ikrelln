use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
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
            ::DB_EXECUTOR_POOL.send(::db::scripts::SaveScript(new_script.clone()));
            if let ::engine::streams::ScriptType::StreamTest = new_script.script_type {
                actix::Arbiter::system_registry()
                    .get::<::engine::streams::Streamer>()
                    .send(::engine::streams::AddScript(new_script.clone()));
            }
            return Ok(httpcodes::HTTPOk.build().json(new_script)?);
        })
        .responder()
}

pub fn get_script(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    match req.match_info().get("scriptId") {
        Some(script_id) => ::DB_EXECUTOR_POOL
            .call_fut(::db::scripts::GetScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Ok(Some(script)) => Ok(httpcodes::HTTPOk.build().json(script)?),
                Ok(None) => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
                Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
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
            .call_fut(::db::scripts::DeleteScript(script_id.to_string()))
            .from_err()
            .and_then(|res| match res {
                Ok(Some(script)) => {
                    if let ::engine::streams::ScriptType::StreamTest = script.script_type {
                        actix::Arbiter::system_registry()
                            .get::<::engine::streams::Streamer>()
                            .send(::engine::streams::RemoveScript(script.clone()));
                    }

                    Ok(httpcodes::HTTPOk.build().json(script)?)
                }
                Ok(None) => Err(super::errors::IkError::NotFound(
                    "script not found".to_string(),
                )),
                Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
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
        .call_fut(::db::scripts::GetAll)
        .from_err()
        .and_then(|res| match res {
            Ok(scripts) => Ok(httpcodes::HTTPOk.build().json(scripts)?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}
