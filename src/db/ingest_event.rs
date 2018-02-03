use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use chrono;

use db::schema::ingest;
#[derive(Debug, Insertable)]
#[table_name = "ingest"]
pub struct IngestEventDb {
    id: String,
    created_at: chrono::NaiveDateTime,
    processed_at: Option<chrono::NaiveDateTime>,
}
impl ResponseType for IngestEventDb {
    type Item = ();
    type Error = ();
}

impl<'a, T> From<&'a ::engine::ingestor::IngestEvents<T>> for IngestEventDb {
    fn from(ie: &::engine::ingestor::IngestEvents<T>) -> IngestEventDb {
        IngestEventDb {
            id: ie.ingest_id.to_string(),
            created_at: ie.created_at,
            processed_at: ie.processed_at,
        }
    }
}
impl Handler<IngestEventDb> for super::DbExecutor {
    type Result = MessageResult<IngestEventDb>;

    fn handle(&mut self, msg: IngestEventDb, _: &mut Self::Context) -> Self::Result {
        use super::schema::ingest::dsl::*;

        let ingest_id = msg.id.clone();
        let insert = diesel::insert_into(ingest).values(&msg).execute(&self.0);
        if let Err(_) = insert {
            diesel::update(ingest)
                .filter(id.eq(msg.id))
                .set(processed_at.eq(msg.processed_at))
                .execute(&self.0)
                .expect(&format!("Error updating Ingest"));
            info!("finishing ingest '{}'", ingest_id);
        } else {
            info!("starting ingest '{}'", ingest_id);
        }

        Ok(())
    }
}

pub struct IngestCleanup(pub chrono::NaiveDateTime);
impl ResponseType for IngestCleanup {
    type Item = ();
    type Error = ();
}
impl Handler<IngestCleanup> for super::DbExecutor {
    type Result = MessageResult<IngestCleanup>;

    fn handle(&mut self, msg: IngestCleanup, _: &mut Self::Context) -> Self::Result {
        use super::schema::ingest::dsl::*;

        diesel::delete(ingest.filter(processed_at.lt(msg.0)))
            .execute(&self.0)
            .expect("Error cleaning up Ingest");

        Ok(())
    }
}
