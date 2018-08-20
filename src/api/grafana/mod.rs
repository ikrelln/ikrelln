use actix_web::*;

use super::{errors, AppState};

pub mod search;
pub use self::search::search;
pub mod query;
pub use self::query::query;
pub mod data_queries;

pub fn setup(_req: &HttpRequest<AppState>) -> String {
    String::from(::engine::hello())
}
