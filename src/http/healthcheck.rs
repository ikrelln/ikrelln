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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZipkinUiConfig {
    environment: String,
    query_limit: u16,
    default_lookback: u32,
    instrumented: String,
    logs_url: Option<String>,
    search_enabled: bool,
    dependency: DependencyErrorRates,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DependencyErrorRates {
    low_error_rate: f32,
    high_error_rate: f32,
}

pub fn zipkin_ui_config(_: HttpRequest<AppState>) -> HttpResponse {
    httpcodes::HTTPOk
        .build()
        .json(ZipkinUiConfig {
            environment: "".to_string(),
            query_limit: 10000,
            default_lookback: 3600000,
            instrumented: ".*".to_string(),
            logs_url: None,
            search_enabled: true,
            dependency: DependencyErrorRates {
                low_error_rate: 0.5,
                high_error_rate: 0.75,
            },
        })
        .unwrap()
}
