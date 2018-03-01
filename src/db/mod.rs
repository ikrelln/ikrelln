use diesel::prelude::*;
use actix::{Actor, SyncContext};

pub mod schema;
pub mod span;
pub mod test;
pub mod scripts;

mod helper;

#[cfg(feature = "sqlite")]
pub fn establish_connection(database_url: &str) -> SqliteConnection {
    info!("opening connection to DB {}", database_url);
    SqliteConnection::establish(database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
#[cfg(feature = "sqlite")]
pub struct DbExecutor(pub SqliteConnection);

#[cfg(feature = "postgres")]
pub fn establish_connection(database_url: &str) -> PgConnection {
    info!("opening connection to DB {}", database_url);
    PgConnection::establish(database_url).expect(&format!("Error connecting to {}", database_url))
}
#[cfg(feature = "postgres")]
pub struct DbExecutor(pub PgConnection);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}
