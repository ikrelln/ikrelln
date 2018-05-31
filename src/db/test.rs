use actix::{Handler, Message, MessageResult};
use chrono;
use diesel;
use diesel::prelude::*;
use serde_json;
use uuid;

static TEST_ITEM_QUERY_LIMIT: i64 = 200;
use db::schema::test_item;
#[derive(Debug, Insertable, Queryable, Clone, Identifiable)]
#[table_name = "test_item"]
pub struct TestItemDb {
    pub id: String,
    pub parent_id: String,
    pub name: String,
    source: i32,
}

static TEST_RESULT_QUERY_LIMIT: i64 = 100;
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
    pub fn into_i32(&self) -> i32 {
        match self {
            &ResultCleanupStatus::WithData => 0,
            &ResultCleanupStatus::Important => 1,
            &ResultCleanupStatus::Shell => 2,
            &ResultCleanupStatus::ToKeep => 3,
        }
    }
}
impl Into<i32> for ResultCleanupStatus {
    fn into(self) -> i32 {
        self.into_i32()
    }
}

impl super::DbExecutor {
    fn find_test_item(&mut self, test_item_db: &TestItemDb) -> Option<TestItemDb> {
        use super::schema::test_item::dsl::*;

        test_item
            .filter(name.eq(&test_item_db.name))
            .filter(source.eq(test_item_db.source))
            .filter(parent_id.eq(&test_item_db.parent_id))
            .first::<TestItemDb>(self.0.as_ref().unwrap())
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
                    .execute(self.0.as_ref().unwrap());
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

        use super::schema::test_result::dsl::*;
        diesel::insert_into(test_result)
            .values(&TestResultDb {
                test_id: parent_id.clone(),
                trace_id: msg.trace_id.clone(),
                date: test_result_date,
                status: msg.status.into_i32(),
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
            })
            .execute(self.0.as_ref().unwrap())
            .map_err(|err| self.reconnect_if_needed(ctx, err))
            .ok();

        diesel::update(
            test_result
                .filter(cleanup_status.eq(super::test::ResultCleanupStatus::ToKeep.into_i32()))
                .filter(test_id.eq(parent_id.clone()))
                .filter(date.lt(test_result_date)),
        ).set(cleanup_status.eq(super::test::ResultCleanupStatus::WithData.into_i32()))
            .execute(self.0.as_ref().unwrap())
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

pub struct GetTestItems(pub TestItemQuery);
impl Message for GetTestItems {
    type Result = Vec<::api::test::TestDetails>;
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

        let mut test_item_cache = super::helper::Cacher::new();

        MessageResult(
            query
                .order(name.asc())
                .load::<TestItemDb>(self.0.as_ref().unwrap())
                .unwrap_or_else(|err| {
                    error!("error loading test items: {:?}", err);
                    vec![]
                })
                .iter()
                .map(|ti| {
                    let mut test_item_to_get = match ti.parent_id.as_ref() {
                        "root" => None,
                        item_id => Some(item_id.to_string()),
                    };
                    let mut path = vec![];
                    if msg.0.with_full_path {
                        while test_item_to_get.is_some() {
                            if let Some(test) = test_item_cache
                                .get(&test_item_to_get.unwrap(), |ti_id| {
                                    use super::schema::test_item::dsl::*;
                                    test_item
                                        .filter(id.eq(ti_id))
                                        .first::<TestItemDb>(self.0.as_ref().unwrap())
                                        .ok()
                                })
                                .clone()
                            {
                                test_item_to_get = match test.parent_id.as_ref() {
                                    "root" => None,
                                    item_id => Some(item_id.to_string()),
                                };
                                path.push(::api::test::TestItem {
                                    id: test.id.clone(),
                                    name: test.name.clone(),
                                });
                            } else {
                                test_item_to_get = None;
                            }
                        }
                        path.reverse();
                    }

                    let children = if msg.0.with_children {
                        use super::schema::test_item::dsl::*;
                        test_item
                            .filter(parent_id.eq(&ti.id))
                            .order(name.asc())
                            .limit(TEST_ITEM_QUERY_LIMIT)
                            .load::<TestItemDb>(self.0.as_ref().unwrap())
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|ti| ::api::test::TestItem {
                                name: ti.name.clone(),
                                id: ti.id.clone(),
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    let traces = if msg.0.with_traces {
                        use super::schema::test_result::dsl::*;

                        let query = TestResultDb::belonging_to(ti).order(date.desc()).limit(5);
                        query
                            .load::<TestResultDb>(self.0.as_ref().unwrap())
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|tr| ::engine::test_result::TestResult {
                                test_id: tr.test_id.clone(),
                                path: path.iter().map(|ti| ti.name.clone()).collect(),
                                name: ti.name.clone(),
                                date: (((tr.date.timestamp() * 1000)
                                    + i64::from(tr.date.timestamp_subsec_millis()))
                                    * 1000),
                                duration: tr.duration,
                                environment: tr.environment.clone(),
                                status: tr.status.into(),
                                trace_id: tr.trace_id.clone(),
                                components_called: serde_json::from_str(&tr.components_called)
                                    .unwrap(),
                                nb_spans: tr.nb_spans,
                                main_span: None,
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    ::api::test::TestDetails {
                        children,
                        last_results: traces,
                        name: ti.name.clone(),
                        path,
                        test_id: ti.id.clone(),
                    }
                })
                .collect(),
        )
    }
}
#[derive(Debug)]
pub struct TestResultQuery {
    pub trace_id: Option<String>,
    pub status: Option<i32>,
    pub test_id: Option<String>,
    pub environment: Option<String>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub ts: chrono::NaiveDateTime,
    pub lookback: Option<chrono::Duration>,
    pub limit: i64,
}

impl Default for TestResultQuery {
    fn default() -> Self {
        TestResultQuery {
            trace_id: None,
            status: None,
            test_id: None,
            environment: None,
            min_duration: None,
            max_duration: None,
            ts: chrono::Utc::now().naive_utc(),
            lookback: None,
            limit: TEST_RESULT_QUERY_LIMIT,
        }
    }
}

impl From<::api::test::TestResultsQueryParams> for TestResultQuery {
    fn from(params: ::api::test::TestResultsQueryParams) -> Self {
        TestResultQuery {
            trace_id: params.trace_id,
            status: params.status.and_then(|v| match v {
                ::engine::test_result::TestStatus::Any => None,
                v => Some(v.into()),
            }),
            test_id: params.test_id,
            environment: params.environment,
            min_duration: params.min_duration,
            max_duration: params.max_duration,
            ts: params
                .ts
                .map(|v| {
                    // query timestamp is in milliseconds
                    chrono::NaiveDateTime::from_timestamp(
                        v / 1000,
                        ((v % 1000) * 1000 * 1000) as u32,
                    )
                })
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            lookback: params.lookback.map(chrono::Duration::milliseconds),
            limit: params
                .limit
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
impl Message for GetTestResults {
    type Result = Vec<::engine::test_result::TestResult>;
}
impl Handler<GetTestResults> for super::DbExecutor {
    type Result = MessageResult<GetTestResults>;

    fn handle(&mut self, msg: GetTestResults, ctx: &mut Self::Context) -> Self::Result {
        self.check_db_connection(ctx);
        use super::schema::test_result::dsl::*;

        let mut query = test_result.into_boxed();

        if let Some(query_trace_id) = msg.0.trace_id {
            query = query.filter(trace_id.eq(query_trace_id));
        }

        if let Some(query_status) = msg.0.status {
            query = query.filter(status.eq(query_status));
        }

        if let Some(query_test_id) = msg.0.test_id {
            query = query.filter(test_id.eq(query_test_id));
        }

        if let Some(query_environment) = msg.0.environment {
            query = query.filter(environment.eq(query_environment));
        }

        if let Some(query_max_duration) = msg.0.max_duration {
            query = query.filter(duration.le(query_max_duration));
        }
        if let Some(query_min_duration) = msg.0.min_duration {
            query = query.filter(duration.ge(query_min_duration));
        }

        query = query.filter(date.le(msg.0.ts));
        if let Some(query_lookback) = msg.0.lookback {
            query = query.filter(date.ge(msg.0.ts - query_lookback));
        }

        let test_results: Vec<TestResultDb> = query
            .order(date.desc())
            .limit(msg.0.limit)
            .load(self.0.as_ref().unwrap())
            .unwrap_or_else(|err| {
                error!("error loading test results: {:?}", err);
                self.reconnect_if_needed(ctx, err);
                vec![]
            });

        let mut test_item_cache = super::helper::Cacher::new_with({
            //prefetch first level test items in one query
            use super::schema::test_item::dsl::*;

            let mut query = test_item.into_boxed();
            for tr in &test_results {
                query = query.or_filter(id.eq(&tr.test_id));
            }
            query
                .load::<TestItemDb>(self.0.as_ref().unwrap())
                .ok()
                .unwrap_or_else(|| vec![])
                .iter()
                .map(|item| (item.id.clone(), Some(item.clone())))
                .collect()
        });

        MessageResult(
            test_results
                .iter()
                .map(|tr| {
                    let test = test_item_cache
                        .get(&tr.test_id, |ti_id| {
                            use super::schema::test_item::dsl::*;

                            test_item
                                .filter(id.eq(ti_id))
                                .first::<TestItemDb>(self.0.as_ref().unwrap())
                                .ok()
                        })
                        .clone();

                    let mut test_item_to_get =
                        test.clone().and_then(|t| match t.parent_id.as_ref() {
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
                                    .first::<TestItemDb>(self.0.as_ref().unwrap())
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

                    ::engine::test_result::TestResult {
                        test_id: tr.test_id.clone(),
                        path,
                        name: test.unwrap().name,
                        date: (((tr.date.timestamp() * 1000)
                            + i64::from(tr.date.timestamp_subsec_millis()))
                            * 1000),
                        duration: tr.duration,
                        environment: tr.environment.clone(),
                        status: tr.status.into(),
                        trace_id: tr.trace_id.clone(),
                        components_called: serde_json::from_str(&tr.components_called).unwrap(),
                        nb_spans: tr.nb_spans,
                        main_span: None,
                    }
                })
                .collect::<Vec<::engine::test_result::TestResult>>(),
        )
    }
}

pub struct GetEnvironments;
impl Message for GetEnvironments {
    type Result = Vec<String>;
}
impl Handler<GetEnvironments> for super::DbExecutor {
    type Result = MessageResult<GetEnvironments>;

    fn handle(&mut self, _msg: GetEnvironments, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_result::dsl::*;

        MessageResult(
            test_result
                .select(environment)
                .filter(environment.is_not_null())
                .distinct()
                .load::<Option<String>>(self.0.as_ref().unwrap())
                .unwrap_or_else(|err| {
                    error!("error loading environment from test results: {:?}", err);
                    vec![]
                })
                .iter()
                .map(|v| v.clone().unwrap())
                .collect(),
        )
    }
}
