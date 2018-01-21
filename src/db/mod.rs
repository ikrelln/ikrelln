use diesel::prelude::*;
use actix::{Actor, SyncContext};

pub mod schema;
pub mod ingest_event;
pub mod test_result;

pub fn establish_connection(database_url: String) -> SqliteConnection {
    info!("opening connection to DB {}", database_url);
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub struct DbExecutor(pub SqliteConnection);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
