use actix_web::{middleware, Application, HttpRequest, HttpServer, Method};
use engine;
use uuid;
use chrono;

mod healthcheck;
mod errors;
mod span;

fn index(_req: HttpRequest<AppState>) -> String {
    String::from(engine::hello())
}

pub struct AppState {
    start_time: chrono::DateTime<chrono::Utc>,
}

pub fn serve(host: &str, port: u16) {
    HttpServer::new(move || {
        Application::with_state(AppState {
            start_time: chrono::Utc::now(),
        }).middleware(
            middleware::DefaultHeaders::build()
                .header(
                    "X-Request-Id",
                    uuid::Uuid::new_v4().hyphenated().to_string().as_str(),
                )
                .finish(),
        )
            .middleware(middleware::Logger::new(
                "%a %t \"%r\" %s %b \"%{Referer}i\" \"%{User-Agent}i\" %{X-Request-Id}o - %T",
            ))
            .resource("/", |r| r.method(Method::GET).f(index))
            .resource("/healthcheck", |r| {
                r.method(Method::GET).f(healthcheck::healthcheck)
            })
            .resource("/config.json", |r| {
                r.method(Method::GET).f(healthcheck::zipkin_ui_config)
            })
            .resource("/api/v1/spans", |r| {
                r.method(Method::POST).f(span::ingest);
                r.method(Method::GET).f(span::get_spans_by_service);
            })
            .resource("/api/v1/services", |r| {
                r.method(Method::GET).f(span::get_services)
            })
            .resource("/api/v1/trace/{traceId}", |r| {
                r.method(Method::GET).f(span::get_spans_by_trace_id)
            })
            .resource("/api/v1/traces", |r| {
                r.method(Method::GET).f(span::get_traces)
            })
            .resource("/api/v1/dependencies", |r| {
                r.method(Method::GET).f(span::get_dependencies)
            })
    }).bind(format!("{}:{}", host, port))
        .unwrap()
        .start();
}
