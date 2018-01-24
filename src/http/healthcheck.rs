use actix_web::{httpcodes, HttpRequest, HttpResponse};
use super::AppState;

#[derive(Debug, Serialize)]
pub struct HealthcheckResponse {
    app_name: &'static str,
    build_info: ::build_info::BuildInfo,
}

pub fn healthcheck(_: HttpRequest<AppState>) -> HttpResponse {
    httpcodes::HTTPOk
        .build()
        .json(HealthcheckResponse {
            app_name: "i'Krelln",
            build_info: ::build_info::BUILD_INFO.clone(),
        })
        .unwrap()
}
