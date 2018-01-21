use diesel;
use diesel::prelude::*;
use actix::prelude::*;

pub mod schema;

pub fn establish_connection(database_url: String) -> SqliteConnection {
    info!("opening connection to DB {}", database_url);
    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub struct DbExecutor(pub SqliteConnection);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

impl Handler<::engine::ingestor::IngestEvents> for DbExecutor {
    type Result = MessageResult<::engine::ingestor::IngestEvents>;

    fn handle(
        &mut self,
        msg: ::engine::ingestor::IngestEvents,
        _: &mut Self::Context,
    ) -> Self::Result {
        use self::schema::ingest::dsl::*;

        let ingest_id = msg.ingest_id.clone();
        diesel::insert_into(ingest)
            .values(&::engine::ingestor::IngestEventDb::from(msg))
            .execute(&self.0)
            .expect("Error inserting Ingest");
        info!("done saving  ingest '{}'", ingest_id);
        Ok(())
    }
}
