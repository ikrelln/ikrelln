use actix::{Handler, Message, MessageResult};
use chrono;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid;

use db::schema::test_item;
#[derive(Debug, Insertable, Queryable, Clone, Identifiable)]
#[table_name = "test_item"]
pub struct TestItemDb {
    pub id: String,
    pub parent_id: String,
    pub name: String,
    source: i32,
}

use db::schema::test_result;
#[derive(Debug, Insertable, Queryable, Associations, Identifiable)]
#[belongs_to(TestItemDb, foreign_key = "test_id")]
#[primary_key(test_id, trace_id)]
#[table_name = "test_result"]
pub struct TestResultDb {
    pub test_id: String,
    pub trace_id: String,
    pub date: chrono::NaiveDateTime,
    pub status: i32,
    pub duration: i64,
    pub environment: Option<String>,
    pub components_called: String,
    pub nb_spans: i32,
    pub cleanup_status: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ResultCleanupStatus {
    WithData,
    Important,
    ToKeep,
    Shell,
}
impl From<i32> for ResultCleanupStatus {
    fn from(v: i32) -> Self {
        match v {
            1 => ResultCleanupStatus::Important,
            2 => ResultCleanupStatus::Shell,
            3 => ResultCleanupStatus::ToKeep,
            _ => ResultCleanupStatus::WithData,
        }
    }
}
impl ResultCleanupStatus {
    pub fn as_i32(&self) -> i32 {
        match self {
            ResultCleanupStatus::WithData => 0,
            ResultCleanupStatus::Important => 1,
            ResultCleanupStatus::Shell => 2,
            ResultCleanupStatus::ToKeep => 3,
        }
    }
}
impl Into<i32> for ResultCleanupStatus {
    fn into(self) -> i32 {
        self.as_i32()
    }
}

impl super::DbExecutor {
    fn find_test_item(&mut self, test_item_db: &TestItemDb) -> Option<TestItemDb> {
        use super::super::schema::test_item::dsl::*;

        test_item
            .filter(name.eq(&test_item_db.name))
            .filter(source.eq(test_item_db.source))
            .filter(parent_id.eq(&test_item_db.parent_id))
            .first::<TestItemDb>(self.0.as_ref().expect("fail to get DB"))
            .ok()
    }

    fn find_test_or_insert(&mut self, test_item_db: &TestItemDb) -> String {
        use super::super::schema::test_item::dsl::*;

        match self.find_test_item(test_item_db) {
            Some(existing) => existing.id,
            None => {
                let new_id = uuid::Uuid::new_v4().to_hyphenated().to_string();
                let could_insert = diesel::insert_into(test_item)
                    .values(&TestItemDb {
                        id: new_id.clone(),
                        ..(*test_item_db).clone()
                    }).execute(self.0.as_ref().expect("fail to get DB"));
                if could_insert.is_err() {
                    self.find_test_item(test_item_db)
                        .map(|existing| existing.id)
                        .expect("should have found an test ID after insertion failed")
                } else {
                    new_id
                }
            }
        }
    }
}

impl Message for ::engine::test_result::TestResult {
    type Result = ::engine::test_result::TestResult;
}

impl Handler<::engine::test_result::TestResult> for super::DbExecutor {
    type Result = MessageResult<::engine::test_result::TestResult>;

    fn handle(
        &mut self,
        msg: ::engine::test_result::TestResult,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        self.check_db_connection(ctx);

        let mut parent_id = "root".to_string();
        for item in msg.path.clone() {
            parent_id = self.find_test_or_insert(&TestItemDb {
                id: "n/a".to_string(),
                parent_id,
                name: item,
                source: 0,
            });
        }

        parent_id = self.find_test_or_insert(&TestItemDb {
            id: "n/a".to_string(),
            parent_id,
            name: msg.name.clone(),
            source: 0,
        });

        let test_result_date = chrono::NaiveDateTime::from_timestamp(
            msg.date / 1000 / 1000,
            (msg.date % (1000 * 1000) * 1000) as u32,
        );

        use super::super::schema::test_result::dsl::*;
        diesel::insert_into(test_result)
            .values(&TestResultDb {
                test_id: parent_id.clone(),
                trace_id: msg.trace_id.clone(),
                date: test_result_date,
                status: msg.status.as_i32(),
                duration: msg.duration,
                environment: msg.environment.clone(),
                components_called: serde_json::to_string(&msg.components_called).unwrap(),
                nb_spans: msg.nb_spans,
                cleanup_status: match msg.status {
                    ::engine::test_result::TestStatus::Success => {
                        ResultCleanupStatus::ToKeep.into()
                    }
                    _ => ResultCleanupStatus::WithData.into(),
                },
            }).execute(self.0.as_ref().expect("fail to get DB"))
            .map_err(|err| self.reconnect_if_needed(ctx, &err))
            .ok();

        diesel::update(
            test_result
                .filter(cleanup_status.eq(super::test::ResultCleanupStatus::ToKeep.as_i32()))
                .filter(test_id.eq(parent_id.clone()))
                .filter(date.lt(test_result_date)),
        ).set(cleanup_status.eq(super::test::ResultCleanupStatus::WithData.as_i32()))
        .execute(self.0.as_ref().expect("fail to get DB"))
        .ok();

        MessageResult(::engine::test_result::TestResult {
            test_id: parent_id,
            ..msg
        })
    }
}

#[derive(Default)]
pub struct TestItemQuery {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub with_full_path: bool,
    pub with_children: bool,
    pub with_traces: bool,
}
