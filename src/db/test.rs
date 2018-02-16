use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use chrono;
use actix_web;

use db::schema::test_item;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "test_item"]
pub struct TestItemDb {
    pub id: String,
    parent_id: Option<String>,
    pub name: String,
    source: i32,
}

use db::schema::test_result;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "test_result"]
pub struct TestResultDb {
    test_id: String,
    pub trace_id: String,
    pub date: chrono::NaiveDateTime,
    pub status: i32,
    pub duration: i64,
    pub environment: Option<String>,
}

impl super::DbExecutor {
    fn find_test_item(&mut self, test_item_db: &TestItemDb) -> Option<TestItemDb> {
        use super::schema::test_item::dsl::*;

        let mut query = test_item
            .filter(name.eq(test_item_db.name.clone()))
            .filter(source.eq(test_item_db.source))
            .into_boxed();

        query = match test_item_db.parent_id.clone() {
            Some(filter_parent_id) => query.filter(parent_id.eq(filter_parent_id)),
            None => query.filter(parent_id.is_null()),
        };

        query.first::<TestItemDb>(&self.0).ok()
    }

    fn find_test_or_insert(&mut self, test_item_db: &TestItemDb) -> String {
        use super::schema::test_item::dsl::*;

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

impl ResponseType for ::engine::test::TestResult {
    type Item = ::engine::test::TestResult;
    type Error = ();
}

impl Handler<::engine::test::TestResult> for super::DbExecutor {
    type Result = MessageResult<::engine::test::TestResult>;

    fn handle(&mut self, msg: ::engine::test::TestResult, _: &mut Self::Context) -> Self::Result {
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

        use super::schema::test_result::dsl::*;
        diesel::insert_into(test_result)
            .values(&TestResultDb {
                test_id: parent_id.unwrap(),
                trace_id: msg.trace_id.clone(),
                date: chrono::NaiveDateTime::from_timestamp(
                    msg.date / 1000 / 1000,
                    (msg.date % (1000 * 1000) * 1000) as u32,
                ),
                status: match msg.status {
                    ::engine::test::TestStatus::Success => 0,
                    ::engine::test::TestStatus::Failure => 1,
                    ::engine::test::TestStatus::Skipped => 2,
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

#[derive(Debug)]
pub struct TestResultQuery {
    pub status: Option<i32>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub end_ts: chrono::NaiveDateTime,
    pub lookback: Option<chrono::Duration>,
    pub limit: i64,
    pub only_endpoint: bool,
}

impl Default for TestResultQuery {
    fn default() -> Self {
        TestResultQuery {
            status: None,
            min_duration: None,
            max_duration: None,
            end_ts: chrono::Utc::now().naive_utc(),
            lookback: None,
            limit: 1000,
            only_endpoint: false,
        }
    }
}

impl TestResultQuery {
    pub fn from_req(req: &actix_web::HttpRequest<::api::AppState>) -> Self {
        TestResultQuery {
            status: req.query().get("status").and_then(|status| {
                match status.to_lowercase().as_ref() {
                    "success" => Some(0),
                    "failure" => Some(1),
                    "skipped" => Some(2),
                    _ => None,
                }
            }),
            min_duration: req.query()
                .get("minDuration")
                .and_then(|s| s.parse::<i64>().ok()),
            max_duration: req.query()
                .get("maxDuration")
                .and_then(|s| s.parse::<i64>().ok()),
            end_ts: req.query()
                .get("endTs")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|v| {
                    // query timestamp is in milliseconds
                    chrono::NaiveDateTime::from_timestamp(
                        v / 1000,
                        ((v % 1000) * 1000 * 1000) as u32,
                    )
                })
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            lookback: req.query()
                .get("lookback")
                .and_then(|s| s.parse::<i64>().ok())
                .map(chrono::Duration::milliseconds),
            limit: req.query()
                .get("limit")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|v| if v > 1000 { 1000 } else { v })
                .unwrap_or(1000),
            only_endpoint: false,
        }
    }
}

pub struct GetTestResults(pub TestResultQuery);
impl ResponseType for GetTestResults {
    type Item = Vec<::engine::test::TestResult>;
    type Error = ();
}
impl Handler<GetTestResults> for super::DbExecutor {
    type Result = MessageResult<GetTestResults>;

    fn handle(&mut self, msg: GetTestResults, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_result::dsl::*;

        let mut query = test_result.into_boxed();

        if let Some(query_status) = msg.0.status {
            query = query.filter(status.eq(query_status));
        }

        if let Some(query_max_duration) = msg.0.max_duration {
            query = query.filter(duration.le(query_max_duration));
        }
        if let Some(query_min_duration) = msg.0.min_duration {
            query = query.filter(duration.ge(query_min_duration));
        }

        query = query.filter(date.le(msg.0.end_ts));
        if let Some(query_lookback) = msg.0.lookback {
            query = query.filter(date.ge(msg.0.end_ts - query_lookback));
        }

        let test_results: Vec<TestResultDb> =
            query.load(&self.0).expect("error loading test results");

        Ok(test_results
            .iter()
            .map(|tr| ::engine::test::TestResult {
                path: vec![],
                name: "".to_string(),
                date: (((tr.date.timestamp() * 1000) + i64::from(tr.date.timestamp_subsec_millis()))
                    * 1000),
                duration: tr.duration,
                environment: tr.environment.clone(),
                status: match tr.status {
                    0 => ::engine::test::TestStatus::Success,
                    1 => ::engine::test::TestStatus::Failure,
                    2 => ::engine::test::TestStatus::Skipped,
                    _ => ::engine::test::TestStatus::Failure,
                },
                trace_id: tr.trace_id.clone(),
            })
            .collect::<Vec<::engine::test::TestResult>>())
    }
}
