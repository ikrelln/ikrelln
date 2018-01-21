use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;

use db::schema::ingest;
#[derive(Debug, Insertable)]
#[table_name = "ingest"]
pub struct StartIngestEventDb {
    id: String,
    created_at: String,
}
impl ResponseType for StartIngestEventDb {
    type Item = ();
    type Error = ();
}

impl<'a> From<&'a ::engine::ingestor::IngestEvents> for StartIngestEventDb {
    fn from(ie: &::engine::ingestor::IngestEvents) -> StartIngestEventDb {
        StartIngestEventDb {
            id: ie.ingest_id.to_string(),
            created_at: ie.created_at.to_rfc2822(),
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "ingest"]
pub struct FinishIngestEventDb {
    id: String,
    processed_at: String,
}
impl ResponseType for FinishIngestEventDb {
    type Item = ();
    type Error = ();
}

impl<'a> From<&'a ::engine::ingestor::IngestEvents> for FinishIngestEventDb {
    fn from(ie: &::engine::ingestor::IngestEvents) -> FinishIngestEventDb {
        FinishIngestEventDb {
            id: ie.ingest_id.to_string(),
            processed_at: ie.processed_at
                .map(|date| date.to_rfc2822())
                .unwrap_or("N/A".to_string()),
        }
    }
}


impl Handler<StartIngestEventDb> for super::DbExecutor {
    type Result = MessageResult<StartIngestEventDb>;

    fn handle(&mut self, msg: StartIngestEventDb, _: &mut Self::Context) -> Self::Result {
        use super::schema::ingest::dsl::*;

        let ingest_id = msg.id.clone();
        diesel::insert_into(ingest)
            .values(&msg)
            .execute(&self.0)
            .expect("Error inserting Ingest");
        info!("done saving  ingest '{}'", ingest_id);
        Ok(())
    }
}

impl Handler<FinishIngestEventDb> for super::DbExecutor {
    type Result = MessageResult<FinishIngestEventDb>;

    fn handle(&mut self, msg: FinishIngestEventDb, _: &mut Self::Context) -> Self::Result {
        use super::schema::ingest::dsl::*;

        let ingest_id = msg.id.clone();
        diesel::update(ingest.find(msg.id))
            .set(processed_at.eq(msg.processed_at))
            .execute(&self.0)
            .expect("Error inserting Ingest");
        info!("done saving  ingest '{}'", ingest_id);
        Ok(())
    }
}
