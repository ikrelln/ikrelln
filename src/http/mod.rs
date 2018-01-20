use actix_web::*;
use engine;
use futures::Future;
use uuid;
use std;
use serde;

fn index(_req: HttpRequest) -> String {
    String::from(engine::hello())
}

#[derive(Deserialize, Serialize, Debug)]
enum Status {
    SUCCESS,
    FAILURE,
    SKIPPED,
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

#[derive(Deserialize, Serialize, Debug)]
struct TestResult {
    test_name: String,
    result: Status,
    #[serde(deserialize_with = "deserialize_duration")] duration: std::time::Duration,
}

use serde::de::{self, Deserialize, MapAccess, Visitor};
fn deserialize_duration<'de, D>(
    deserializer: D,
) -> std::result::Result<std::time::Duration, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct IntOrStruct(std::marker::PhantomData<fn() -> std::time::Duration>);

    impl<'de> Visitor<'de> for IntOrStruct {
        type Value = std::time::Duration;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("int or map")
        }

        fn visit_u64<E>(self, value: u64) -> Result<std::time::Duration, E>
        where
            E: de::Error,
        {
            Ok(std::time::Duration::new(value, 0))
        }

        fn visit_map<M>(self, visitor: M) -> Result<std::time::Duration, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(IntOrStruct(std::marker::PhantomData))
}

fn ingest(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = IkError>> {
    req.json()
        .from_err()
        .and_then(|val: Vec<TestResult>| {
            info!("model: {:?}", val);
            Ok(httpcodes::HTTPOk.build().json(val)?)
        })
        .responder()
}

pub fn serve(port: u16) {
    HttpServer::new(|| {
        Application::new()
            .middleware(
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
            .resource("/ingest", |r| r.method(Method::POST).f(ingest))
    }).bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run();
}
