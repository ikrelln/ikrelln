use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use chrono;

use db::schema::test_item;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "test_item"]
pub struct TestItemDb {
    pub id: String,
    parent_id: Option<String>,
    pub name: String,
    source: i32,
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
    fn find_test_item(&mut self, test_item_db: &TestItemDb) -> Option<TestItemDb> {
        use super::schema::test_item::dsl::*;

        info!("find_test_item input: {:?}", test_item_db);

        let mut query = test_item
            .filter(name.eq(test_item_db.name.clone()))
            .filter(source.eq(test_item_db.source))
            .into_boxed();

        query = match test_item_db.parent_id.clone() {
            Some(filter_parent_id) => query.filter(parent_id.eq(filter_parent_id)),
            None => query.filter(parent_id.is_null()),
        };

        let a = query.first::<TestItemDb>(&self.0).ok();
        info!("find_test_item result: {:?}", a);
        a
    }

    fn find_test_or_insert(&mut self, test_item_db: &TestItemDb) -> String {
        use super::schema::test_item::dsl::*;
        info!("find_test_or_insert input: {:?}", test_item_db);

        match self.find_test_item(test_item_db) {
            Some(existing) => existing.id,
            None => {
                let new_id = uuid::Uuid::new_v4().hyphenated().to_string();
                let could_insert = diesel::insert_into(test_item)
                    .values(&TestItemDb {
                        id: new_id.clone(),
                        ..(*test_item_db).clone()
                    })
                    .execute(&self.0);
                if could_insert.is_err() {
                    self.find_test_item(test_item_db)
                        .map(|existing| existing.id)
                        .unwrap()
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
        let mut parent_id = None;
        for item in msg.path.clone() {
            parent_id = Some(self.find_test_or_insert(&TestItemDb {
                id: "n/a".to_string(),
                parent_id: parent_id,
                name: item,
                source: 0,
            }));
        }

        parent_id = Some(self.find_test_or_insert(&TestItemDb {
            id: "n/a".to_string(),
            parent_id: parent_id,
            name: msg.name.clone(),
            source: 0,
        }));

        use super::schema::test_execution::dsl::*;
        diesel::insert_into(test_execution)
            .values(&TestExecutionDb {
                test_id: parent_id.unwrap(),
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

pub struct GetTestItems(pub Option<String>);
impl ResponseType for GetTestItems {
    type Item = Vec<TestItemDb>;
    type Error = ();
}

impl Handler<GetTestItems> for super::DbExecutor {
    type Result = MessageResult<GetTestItems>;

    fn handle(&mut self, msg: GetTestItems, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_item::dsl::*;

        let mut query = test_item.into_boxed();

        query = match msg.0.clone() {
            Some(filter_parent_id) => query.filter(parent_id.eq(filter_parent_id)),
            None => query.filter(parent_id.is_null()),
        };

        Ok(query.load(&self.0).expect("error loading test items"))
    }
}
