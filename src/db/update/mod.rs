use actix::{Actor, ActorContext, SyncContext};
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DieselError};

pub mod cleanup;
pub mod reports;
pub mod scripts;
pub mod span;
pub mod test;

#[cfg(feature = "sqlite")]
pub fn establish_connection(database_url: &str) -> ConnectionResult<SqliteConnection> {
    info!("opening connection to DB {}", database_url);
    SqliteConnection::establish(database_url)
}
#[cfg(feature = "sqlite")]
pub struct DbExecutor(pub Option<SqliteConnection>);

#[cfg(feature = "postgres")]
pub fn establish_connection(database_url: &str) -> ConnectionResult<PgConnection> {
    info!("opening connection to DB {}", database_url);
    PgConnection::establish(database_url)
}
#[cfg(feature = "postgres")]
pub struct DbExecutor(pub Option<PgConnection>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl DbExecutor {
    fn check_db_connection(&self, ctx: &mut <Self as Actor>::Context) {
        if self.0.is_none() {
            ctx.stop();
        }
    }

    fn reconnect_if_needed(&self, ctx: &mut <Self as Actor>::Context, error: &DieselError) {
        match error {
            DieselError::DatabaseError(DatabaseErrorKind::UnableToSendCommand, _) => ctx.stop(),
            _ => (),
        }
    }
}
