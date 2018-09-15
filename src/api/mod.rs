use std::net::TcpListener;
use std::os::unix::io::FromRawFd;

use actix_web::middleware::cors;
use actix_web::{http, middleware, server, App, HttpRequest};

use chrono;
use engine;
use uuid;

mod errors;
mod grafana;
mod healthcheck;
pub mod report;
mod script;
pub mod span;
pub mod test;

fn index(_req: &HttpRequest<AppState>) -> String {
    String::from(engine::hello())
}

pub struct AppState {
    start_time: chrono::DateTime<chrono::Utc>,
}

pub fn http_application() -> App<AppState> {
    App::with_state(AppState {
        start_time: chrono::Utc::now(),
    }).middleware(middleware::DefaultHeaders::new().header(
        "X-Request-Id",
        uuid::Uuid::new_v4().hyphenated().to_string().as_str(),
    )).middleware(middleware::Logger::new(
        "%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %{X-Request-Id}o - %T",
    )).middleware(cors::Cors::build().send_wildcard().finish())
    .resource("/", |r| r.method(http::Method::GET).f(index))
    .resource("/healthcheck", |r| {
        r.method(http::Method::GET).f(healthcheck::healthcheck)
    }).resource("/config.json", |r| {
        r.method(http::Method::GET).f(healthcheck::zipkin_ui_config)
    }).resource("/api/v1/spans", |r| {
        r.method(http::Method::POST).f(span::ingest);
        r.method(http::Method::GET).f(span::get_spans_by_service);
    }).resource("/api/v1/services", |r| {
        r.method(http::Method::GET).f(span::get_services)
    }).resource("/api/v1/trace/{traceId}", |r| {
        r.method(http::Method::GET).f(span::get_spans_by_trace_id)
    }).resource("/api/v1/traces", |r| {
        r.method(http::Method::GET).f(span::get_traces)
    }).resource("/api/v1/dependencies", |r| {
        r.method(http::Method::GET).f(span::get_dependencies)
    }).resource("/api/v1/tests", |r| {
        r.method(http::Method::GET).f(test::get_tests_by_parent)
    }).resource("/api/v1/tests/{testId}", |r| {
        r.method(http::Method::GET).f(test::get_test)
    }).resource("/api/v1/testresults", |r| {
        r.method(http::Method::GET).f(test::get_test_results)
    }).resource("/api/v1/environments", |r| {
        r.method(http::Method::GET).f(test::get_environments)
    }).resource("/api/v1/scripts", |r| {
        r.method(http::Method::GET).f(script::list_scripts);
        r.method(http::Method::POST).f(script::save_script);
        r.method(http::Method::PUT).f(script::reload_scripts);
    }).resource("/api/v1/scripts/{scriptId}", |r| {
        r.method(http::Method::GET).f(script::get_script);
        r.method(http::Method::PUT).f(script::update_script);
        r.method(http::Method::DELETE).f(script::delete_script);
    }).resource("/api/v1/reports", |r| {
        r.method(http::Method::GET).f(report::get_reports)
    }).resource("/api/v1/reports/{reportGroup}/{reportName}", |r| {
        r.method(http::Method::GET).f(report::get_report)
    }).resource("/api/grafana/", |r| {
        r.method(http::Method::GET).f(grafana::setup)
    }).resource("/api/grafana/search", |r| {
        r.method(http::Method::POST).f(grafana::search)
    }).resource("/api/grafana/query", |r| {
        r.method(http::Method::POST).f(grafana::query)
    })
}

pub fn serve(host: &str, port: u16) {
    server::new(http_application)
        .bind(format!("{}:{}", host, port))
        .unwrap()
        .start();
}

pub fn serve_from_fd(fd: &str) {
    server::new(http_application)
        .listen(unsafe { TcpListener::from_raw_fd(fd.parse().unwrap()) })
        .start();
}
