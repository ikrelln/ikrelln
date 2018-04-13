use actix_web::*;
use futures::Future;

use super::{errors, AppState};

#[derive(Debug, Deserialize)]
struct Search {
    target: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum SearchResponse {
    Node(String),
    //    Leaf { target: String, value: i32 },
}

pub fn search(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.json()
        .from_err()
        .and_then(move |_val: Search| {
            let resp = vec![
                SearchResponse::Node("spans".to_string()),
                SearchResponse::Node("test_results".to_string()),
                SearchResponse::Node("reports".to_string()),
            ];
            Ok(HttpResponse::Ok().json(resp))
        })
        .responder()
}
