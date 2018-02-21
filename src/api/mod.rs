use actix_web::{middleware, Application, HttpRequest, HttpServer, Method};
use actix_web::middleware::cors;

use engine;
use uuid;
use chrono;

mod healthcheck;
mod errors;
mod span;
pub mod test;
mod script;

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
            .middleware(
                cors::Cors::build()
                    .send_wildcard()
                    .finish()
                    .expect("Error creating CORS middleware"),
            )
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
            .resource("/api/v1/tests", |r| {
                r.method(Method::GET).f(test::get_tests_by_parent)
            })
            .resource("/api/v1/tests/{testId}", |r| {
                r.method(Method::GET).f(test::get_test)
            })
            .resource("/api/v1/testresults", |r| {
                r.method(Method::GET).f(test::get_test_results)
            })
            .resource("/api/v1/scripts", |r| {
                r.method(Method::GET).f(script::list_scripts);
                r.method(Method::POST).f(script::save_script);
            })
            .resource("/api/v1/scripts/{scriptId}", |r| {
                r.method(Method::GET).f(script::get_script);
                r.method(Method::DELETE).f(script::delete_script);
            })
    }).bind(format!("{}:{}", host, port))
        .unwrap()
        .start();
}
