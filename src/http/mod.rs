use actix_web::{middleware, Application, HttpRequest, HttpServer, Method};
use engine;
use uuid;
use actix;
use chrono;

use engine::ingestor::Ingestor;

mod healthcheck;
mod test_result;
mod errors;
mod span;

fn index(_req: HttpRequest<AppState>) -> String {
    String::from(engine::hello())
}

pub struct AppState {
    ingestor: actix::SyncAddress<Ingestor>,
    db_actor: actix::SyncAddress<::db::DbExecutor>,
    start_time: chrono::DateTime<chrono::Utc>,
}

pub fn serve(
    host: String,
    port: u16,
    ingestor: actix::SyncAddress<Ingestor>,
    db_actor: actix::SyncAddress<::db::DbExecutor>,
) {
    HttpServer::new(move || {
        Application::with_state(AppState {
            ingestor: ingestor.clone(),
            db_actor: db_actor.clone(),
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
            .resource("/api/tests", |r| {
                r.method(Method::POST).f(test_result::ingest)
            })
            .resource("/api/spans", |r| r.method(Method::POST).f(span::ingest))
            .resource("/api/services", |r| {
                r.method(Method::GET).f(span::get_services)
            })
            .resource("/api/spans2", |r| {
                r.method(Method::GET).f(span::get_spans_by_service)
            })
    }).bind(format!("{}:{}", host, port))
        .unwrap()
        .start();
}
