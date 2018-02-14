use actix_web::{httpcodes, AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;

use super::{errors, AppState};

#[derive(Serialize)]
struct TestItem {
    id: String,
    name: String,
}

pub fn get_test_by_parent(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    ::DB_EXECUTOR_POOL
        .call_fut(::db::test::GetTestItems(
            req.query().get("parentId").map(|s| s.to_string()),
        ))
        .from_err()
        .and_then(|res| match res {
            Ok(test_items) => Ok(httpcodes::HTTPOk.build().json(
                test_items
                    .iter()
                    .map(|item| {
                        TestItem {
                            id: item.id.clone(),
                            name: item.name.clone(),
                        }
                    })
                    .collect::<Vec<TestItem>>(),
            )?),
            Err(_) => Ok(httpcodes::HTTPInternalServerError.into()),
        })
        .responder()
}
