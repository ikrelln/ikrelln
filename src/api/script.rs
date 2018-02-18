use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
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
                id: Some(uuid::Uuid::new_v4().hyphenated().to_string()),
                status: Some(::engine::streams::ScriptStatus::Enabled),
                date_added: Some(chrono::Utc::now().naive_utc()),
                ..script
            };
            ::DB_EXECUTOR_POOL.send(::db::scripts::SaveScript(new_script.clone()));
            return Ok(httpcodes::HTTPOk.build().json(new_script)?);
        })
        .responder()
}
