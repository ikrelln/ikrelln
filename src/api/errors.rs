use actix;
use actix_web::{error, Error, HttpResponse};
use futures;
use uuid;

#[derive(Fail, Debug, Serialize)]
#[serde(tag = "error", content = "msg")]
pub enum IkError {
    #[fail(display = "internal error")]
    InternalError,
    #[fail(display = "bad request")]
    BadRequest(String),
    #[fail(display = "not found")]
    NotFound(String),
}

impl error::ResponseError for IkError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            IkError::InternalError => {
                let error_uid = uuid::Uuid::new_v4();
                error!("{:?} with id {}", self, error_uid);
                HttpResponse::InternalServerError()
                    .header(
                        "X-Request-Id",
                        error_uid.to_hyphenated().to_string().as_str(),
                    ).finish()
            }
            IkError::BadRequest(_) => HttpResponse::BadRequest().json(self),
            IkError::NotFound(_) => HttpResponse::NotFound().json(self),
        }
    }
}
impl From<error::JsonPayloadError> for IkError {
    fn from(err: error::JsonPayloadError) -> IkError {
        match err {
            error::JsonPayloadError::Deserialize(json_err) => {
                IkError::BadRequest(format!("{}", json_err))
            }
            _ => IkError::BadRequest(format!("{}", err)),
        }
    }
}
impl From<actix::MailboxError> for IkError {
    fn from(err: actix::MailboxError) -> IkError {
        error!("Got a {:?}", err);
        IkError::InternalError
    }
}
impl From<futures::Canceled> for IkError {
    fn from(err: futures::Canceled) -> IkError {
        error!("Got a {:?}", err);
        IkError::InternalError
    }
}
impl From<Error> for IkError {
    fn from(err: Error) -> IkError {
        error!("Got a {:?}", err);
        IkError::InternalError
    }
}
