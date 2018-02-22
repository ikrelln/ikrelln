use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use chrono;
use actix_web;

static TEST_ITEM_QUERY_LIMIT: i64 = 200;
use db::schema::test_item;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "test_item"]
pub struct TestItemDb {
    pub id: String,
    parent_id: String,
    pub name: String,
    source: i32,
}

static TEST_RESULT_QUERY_LIMIT: i64 = 100;
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

        test_item
            .filter(name.eq(test_item_db.name.clone()))
            .filter(source.eq(test_item_db.source))
            .filter(parent_id.eq(test_item_db.parent_id.clone()))
            .first::<TestItemDb>(&self.0)
            .ok()
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
        let mut parent_id = "root".to_string();
        for item in msg.path.clone() {
            parent_id = self.find_test_or_insert(&TestItemDb {
                id: "n/a".to_string(),
                parent_id: parent_id,
                name: item,
                source: 0,
            });
        }

        parent_id = self.find_test_or_insert(&TestItemDb {
            id: "n/a".to_string(),
            parent_id: parent_id,
            name: msg.name.clone(),
            source: 0,
        });

        use super::schema::test_result::dsl::*;
        diesel::insert_into(test_result)
            .values(&TestResultDb {
                test_id: parent_id,
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

#[derive(Default)]
pub struct TestItemQuery {
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub with_full_path: bool,
    pub with_children: bool,
    pub with_traces: bool,
}

pub struct GetTestItems(pub TestItemQuery);
impl ResponseType for GetTestItems {
    type Item = Vec<::api::test::TestDetails>;
    type Error = ();
}

impl Handler<GetTestItems> for super::DbExecutor {
    type Result = MessageResult<GetTestItems>;

    fn handle(&mut self, msg: GetTestItems, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_item::dsl::*;

        let mut query = test_item.into_boxed();

        if let Some(filter_parent_id) = msg.0.parent_id.clone() {
            query = query.filter(parent_id.eq(filter_parent_id));
        }

        if let Some(filter_id) = msg.0.id.clone() {
            query = query.filter(id.eq(filter_id));
        }

        Ok(query
            .order(name.asc())
            .load::<TestItemDb>(&self.0)
            .expect("error loading test items")
            .iter()
            .map(|ti| {
                let mut test_item_to_get = match ti.parent_id.as_ref() {
                    "root" => None,
                    item_id => Some(item_id.to_string()),
                };
                let mut path = vec![];
                if msg.0.with_full_path {
                    while test_item_to_get.is_some() {
                        if let Some(test) = {
                            use super::schema::test_item::dsl::*;
                            test_item
                                .filter(id.eq(test_item_to_get.unwrap()))
                                .first::<TestItemDb>(&self.0)
                                .ok()
                        } {
                            test_item_to_get = Some(test.parent_id);
                            path.push(::api::test::TestItem {
                                name: test.name,
                                id: test.id,
                            });
                        } else {
                            test_item_to_get = None;
                        }
                    }
                    path.reverse();
                }

                let children = match msg.0.with_children {
                    true => {
                        use super::schema::test_item::dsl::*;
                        test_item
                            .filter(parent_id.eq(ti.id.clone()))
                            .order(name.asc())
                            .limit(TEST_ITEM_QUERY_LIMIT)
                            .load::<TestItemDb>(&self.0)
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|ti| ::api::test::TestItem {
                                name: ti.name.clone(),
                                id: ti.id.clone(),
                            })
                            .collect()
                    }
                    false => vec![],
                };

                let traces = match msg.0.with_traces {
                    true => {
                        use super::schema::test_result::dsl::*;

                        test_result
                            .filter(test_id.eq(ti.id.clone()))
                            .order(date.desc())
                            .limit(TEST_RESULT_QUERY_LIMIT)
                            .load::<TestResultDb>(&self.0)
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|tr| tr.trace_id.clone())
                            .collect()
                    }
                    false => vec![],
                };

                ::api::test::TestDetails {
                    children: children,
                    last_traces: traces,
                    name: ti.name.clone(),
                    path: path,
                    test_id: ti.id.clone(),
                }
            })
            .collect())
    }
}
#[derive(Debug)]
pub struct TestResultQuery {
    pub trace_id: Option<String>,
    pub status: Option<i32>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub end_ts: chrono::NaiveDateTime,
    pub lookback: Option<chrono::Duration>,
    pub limit: i64,
}

impl Default for TestResultQuery {
    fn default() -> Self {
        TestResultQuery {
            trace_id: None,
            status: None,
            min_duration: None,
            max_duration: None,
            end_ts: chrono::Utc::now().naive_utc(),
            lookback: None,
            limit: TEST_RESULT_QUERY_LIMIT,
        }
    }
}

impl TestResultQuery {
    pub fn from_req(req: &actix_web::HttpRequest<::api::AppState>) -> Self {
        TestResultQuery {
            trace_id: req.query().get("traceId").map(|s| s.to_string()),
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
                .map(|v| {
                    if v > TEST_RESULT_QUERY_LIMIT {
                        TEST_RESULT_QUERY_LIMIT
                    } else {
                        v
                    }
                })
                .unwrap_or(TEST_RESULT_QUERY_LIMIT),
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

        if let Some(query_trace_id) = msg.0.trace_id {
            query = query.filter(trace_id.eq(query_trace_id));
        }

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

        let test_results: Vec<TestResultDb> = query
            .order(date.desc())
            .limit(msg.0.limit)
            .load(&self.0)
            .expect("error loading test results");

        let mut test_item_cache = super::helper::Cacher::new_with({
            //prefetch first level test items in one query
            use super::schema::test_item::dsl::*;

            let mut query = test_item.into_boxed();
            for tr in test_results.iter() {
                query = query.or_filter(id.eq(tr.test_id.clone()));
            }
            query
                .load::<TestItemDb>(&self.0)
                .ok()
                .unwrap_or_else(|| vec![])
                .iter()
                .map(|item| (item.id.clone(), Some(item.clone())))
                .collect()
        });

        Ok(test_results
            .iter()
            .map(|tr| {
                let test = test_item_cache
                    .get(&tr.test_id, |ti_id| {
                        use super::schema::test_item::dsl::*;

                        test_item
                            .filter(id.eq(ti_id))
                            .first::<TestItemDb>(&self.0)
                            .ok()
                    })
                    .clone();

                let mut test_item_to_get = test.clone().and_then(|t| match t.parent_id.as_ref() {
                    "root" => None,
                    item_id => Some(item_id.to_string()),
                });
                let mut path = vec![];
                while test_item_to_get.is_some() {
                    if let Some(test) = test_item_cache
                        .get(&test_item_to_get.unwrap(), |ti_id| {
                            use super::schema::test_item::dsl::*;
                            test_item
                                .filter(id.eq(ti_id))
                                .first::<TestItemDb>(&self.0)
                                .ok()
                        })
                        .clone()
                    {
                        test_item_to_get = match test.parent_id.as_ref() {
                            "root" => None,
                            item_id => Some(item_id.to_string()),
                        };
                        path.push(test.name);
                    } else {
                        test_item_to_get = None;
                    }
                }
                path.reverse();

                ::engine::test::TestResult {
                    test_id: tr.test_id.clone(),
                    path: path,
                    name: test.unwrap().name,
                    date: (((tr.date.timestamp() * 1000)
                        + i64::from(tr.date.timestamp_subsec_millis()))
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
                }
            })
            .collect::<Vec<::engine::test::TestResult>>())
    }
}
