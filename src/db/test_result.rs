use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;

use db::schema::test_result;

#[derive(Debug, Insertable)]
#[table_name = "test_result"]
pub struct TestResultDb {
    id: String,
    test_id: String,
    result: String,
    duration: i64,
    ts: i64,
}
impl ResponseType for TestResultDb {
    type Item = ();
    type Error = ();
}

impl<'a> From<&'a ::engine::ingestor::TestResult> for TestResultDb {
    fn from(tr: &::engine::ingestor::TestResult) -> TestResultDb {
        TestResultDb {
            id: uuid::Uuid::new_v4().hyphenated().to_string(),
            test_id: tr.test_name.clone(),
            result: format!("{:?}", tr.result),
            duration: tr.duration.as_secs() as i64,
            ts: tr.ts.unwrap_or(0) as i64,
        }
    }
}

impl Handler<TestResultDb> for super::DbExecutor {
    type Result = MessageResult<TestResultDb>;

    fn handle(&mut self, msg: TestResultDb, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_result::dsl::*;

        diesel::insert_into(test_result)
            .values(&msg)
            .execute(&self.0)
            .expect("Error inserting TestResult");
        Ok(())
    }
}
