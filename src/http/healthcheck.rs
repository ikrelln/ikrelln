use actix_web::{httpcodes, HttpRequest, HttpResponse};
use super::AppState;
use chrono;

#[derive(Serialize)]
pub struct HealthcheckResponse {
    app_name: &'static str,
    build_info: ::build_info::BuildInfo,
    time: Times,
}

#[derive(Serialize)]
pub struct Times {
    start_time: chrono::DateTime<chrono::Utc>,
    now: chrono::DateTime<chrono::Utc>,
}

pub fn healthcheck(req: HttpRequest<AppState>) -> HttpResponse {
    httpcodes::HTTPOk
        .build()
        .json(HealthcheckResponse {
            app_name: "i'Krelln",
            build_info: ::build_info::BUILD_INFO.clone(),
            time: Times {
                start_time: req.state().start_time,
                now: chrono::Utc::now(),
            },
        })
        .unwrap()
}

pub fn zipkin_ui_config(_: HttpRequest<AppState>) -> HttpResponse {
    httpcodes::HTTPOk
    .build()
    .body(r#"{"environment":"","queryLimit":10,"defaultLookback":3600000,"instrumented":".*","logsUrl":null,"searchEnabled":true,"dependency":{"lowErrorRate":0.5,"highErrorRate":0.75}}"#).unwrap()
}
