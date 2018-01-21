use actix_web::*;
use engine;
use futures::Future;
use uuid;
use actix;

use engine::ingestor::*;

pub mod healthcheck;

fn index(_req: HttpRequest<AppState>) -> String {
    String::from(engine::hello())
}

#[derive(Fail, Debug, Serialize)]
#[serde(tag = "error", content = "msg")]
enum IkError {
    #[fail(display = "internal error")] InternalError,
    #[fail(display = "bad request")] BadClientData(String),
}

impl error::ResponseError for IkError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            IkError::InternalError => {
                let error_uid = uuid::Uuid::new_v4();
                error!("{:?} with id {}", self, error_uid);
                httpcodes::HTTPInternalServerError
                    .build()
                    .header("X-Request-Id", error_uid.hyphenated().to_string().as_str())
                    .finish()
                    .unwrap()
            }
            IkError::BadClientData(_) => httpcodes::HTTPBadRequest.build().json(self).unwrap(),
        }
    }
}
impl From<error::JsonPayloadError> for IkError {
    fn from(err: error::JsonPayloadError) -> IkError {
        match err {
            error::JsonPayloadError::Deserialize(json_err) => {
                IkError::BadClientData(format!("{}", json_err))
            }
            _ => IkError::BadClientData(format!("{}", err)),
        }
    }
}
impl From<Error> for IkError {
    fn from(_err: Error) -> IkError {
        IkError::InternalError
    }
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum IkResponse {
    IngestResponse {
        ingest_id: ::engine::IngestId,
        nb_events: usize,
    },
}

fn ingest(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = IkError>> {
    let ingestor = req.state().ingestor.clone();
    req.json()
        .from_err()
        .and_then(move |val: Vec<TestResult>| {
            let ingest = NewEvents::new(val.iter().cloned().collect());
            let ingest_id = ingest.ingest_id.clone();
            debug!(
                "ingesting {} event(s) as {}: {:?}",
                val.len(),
                ingest_id,
                val
            );
            ingestor.borrow().send(ingest);
            Ok(httpcodes::HTTPOk.build().json(
                IkResponse::IngestResponse {
                    ingest_id: ingest_id,
                    nb_events: val.len(),
                },
            )?)
        })
        .responder()
}

use std::cell::RefCell;
pub struct AppState {
    ingestor: RefCell<actix::SyncAddress<Ingestor>>,
}

pub fn serve(port: u16, _ingestor: actix::SyncAddress<Ingestor>) {
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
            .resource("/ingest", |r| r.method(Method::POST).f(ingest))
    }).bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .start();
}
