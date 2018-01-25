use actix_web::{middleware, Application, HttpRequest, HttpServer, Method};
use engine;
use uuid;
use actix;
use std::cell::RefCell;

use engine::ingestor::Ingestor;

mod healthcheck;
mod test_result;
mod errors;
mod span;

fn index(_req: HttpRequest<AppState>) -> String {
    String::from(engine::hello())
}

pub struct AppState {
    ingestor: RefCell<actix::SyncAddress<Ingestor>>,
}

pub fn serve(host: String, port: u16, _ingestor: actix::SyncAddress<Ingestor>) {
    HttpServer::new(move || {
        Application::with_state(AppState {
            ingestor: RefCell::new(_ingestor.clone()),
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
    }).bind(format!("{}:{}", host, port))
        .unwrap()
        .start();
}
