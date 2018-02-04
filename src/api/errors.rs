use uuid;
use actix_web::{error, httpcodes, Error, HttpResponse};
use futures;

#[derive(Fail, Debug, Serialize)]
#[serde(tag = "error", content = "msg")]
pub enum IkError {
    #[fail(display = "internal error")] InternalError,
    #[fail(display = "bad request")] BadRequest(String),
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
            IkError::BadRequest(_) => httpcodes::HTTPBadRequest.build().json(self).unwrap(),
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
impl From<futures::Canceled> for IkError {
    fn from(_err: futures::Canceled) -> IkError {
        IkError::InternalError
    }
}
impl From<Error> for IkError {
    fn from(_err: Error) -> IkError {
        IkError::InternalError
    }
}
