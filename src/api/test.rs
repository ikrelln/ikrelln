use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;

use super::{errors, AppState};

pub fn get_test_suites(
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .call_fut(::db::test::GetTestSuites)
        .from_err()
        .and_then(|res| match res {
            Ok(test_suites) => Ok(httpcodes::HTTPOk.build().json(test_suites)?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}
