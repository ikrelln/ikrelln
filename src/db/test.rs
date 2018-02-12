use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use chrono;

use db::schema::test;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "test"]
pub struct TestDb {
    id: String,
    test_suite: String,
    test_class: String,
    test_name: String,
}

use db::schema::test_execution;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "test_execution"]
pub struct TestExecutionDb {
    test_id: String,
    trace_id: String,
    date: chrono::NaiveDateTime,
    result: i32,
    duration: i64,
    environment: Option<String>,
}


impl super::DbExecutor {
    fn find_test(&mut self, testdb: &TestDb) -> Option<TestDb> {
        use super::schema::test::dsl::*;

        test.filter(test_suite.eq(testdb.test_suite.clone()))
            .filter(test_class.eq(testdb.test_class.clone()))
            .filter(test_name.eq(testdb.test_name.clone()))
            .first::<TestDb>(&self.0)
            .ok()
    }

    fn upsert_test(&mut self, testdb: &TestDb) -> String {
        use super::schema::test::dsl::*;

        match self.find_test(testdb) {
            Some(existing) => existing.id,
            None => {
                let new_id = uuid::Uuid::new_v4().hyphenated().to_string();
                let could_insert = diesel::insert_into(test)
                    .values(&TestDb {
                        id: new_id.clone(),
                        test_suite: testdb.test_suite.clone(),
                        test_class: testdb.test_class.clone(),
                        test_name: testdb.test_name.clone(),
                    })
                    .execute(&self.0);
                if could_insert.is_err() {
                    self.find_test(testdb).map(|existing| existing.id).unwrap()
                } else {
                    new_id
                }
            }
        }
    }
}

impl ResponseType for ::engine::test::TestExecution {
    type Item = ::engine::test::TestExecution;
    type Error = ();
}

impl Handler<::engine::test::TestExecution> for super::DbExecutor {
    type Result = MessageResult<::engine::test::TestExecution>;

    fn handle(
        &mut self,
        msg: ::engine::test::TestExecution,
        _: &mut Self::Context,
    ) -> Self::Result {
        let saved_test_id = self.upsert_test(&TestDb {
            id: "n/a".to_string(),
            test_suite: msg.suite.clone(),
            test_class: msg.class.clone(),
            test_name: msg.name.clone(),
        });

        use super::schema::test_execution::dsl::*;
        diesel::insert_into(test_execution)
            .values(&TestExecutionDb {
                test_id: saved_test_id,
                trace_id: msg.trace_id.clone(),
                date: chrono::NaiveDateTime::from_timestamp(
                    msg.date / 1000 / 1000,
                    (msg.date % (1000 * 1000) * 1000) as u32,
                ),
                result: match msg.result {
                    ::engine::test::TestResult::Success => 0,
                    ::engine::test::TestResult::Failure => 1,
                    ::engine::test::TestResult::Skipped => 2,
                },
                duration: msg.duration,
                environment: msg.environment.clone(),
            })
            .execute(&self.0)
            .unwrap();

        Ok(msg)
    }
}
