use std::collections::HashMap;

use actix::prelude::*;
use chrono;
use diesel::prelude::*;
use serde_json;

use engine::test_result::TestStatus;

static REPORT_QUERY_LIMIT: i64 = 200;
use db::schema::report;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "report"]
pub struct ReportDb {
    pub id: String,
    name: String,
    folder: String,
    created_on: chrono::NaiveDateTime,
    last_update: chrono::NaiveDateTime,
}

use db::schema::test_result_in_report;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "test_result_in_report"]
struct TestResultInReportDb {
    report_id: String,
    test_id: String,
    trace_id: String,
    category: String,
    environment: Option<String>,
    status: i32,
}

pub struct GetAll;
impl Message for GetAll {
    type Result = Vec<::api::report::Report>;
}

impl Handler<GetAll> for super::DbReadExecutor {
    type Result = MessageResult<GetAll>;

    fn handle(&mut self, _msg: GetAll, _ctx: &mut Self::Context) -> Self::Result {
        use super::super::schema::report::dsl::*;

        let reports: Vec<ReportDb> = report
            .order(last_update.desc())
            .limit(REPORT_QUERY_LIMIT)
            .load(self.0.as_ref().expect("fail to get DB"))
            .unwrap_or_else(|err| {
                error!("error loading reports: {:?}", err);
                vec![]
            });

        MessageResult(
            reports
                .iter()
                .map(|report_from_db| {
                    let environments: Vec<String> = {
                        use super::super::schema::test_result_in_report::dsl::*;
                        test_result_in_report
                            .select(environment)
                            .filter(report_id.eq(&report_from_db.id))
                            .order(environment.asc())
                            .distinct()
                            .load::<Option<String>>(self.0.as_ref().expect("fail to get DB"))
                            .unwrap_or_else(|err| {
                                error!("error loading environment from reports: {:?}", err);
                                vec![]
                            })
                            .iter()
                            .map(|vo| match vo {
                                Some(ref v) => v.clone(),
                                None => "None".to_string(),
                            })
                            .collect()
                    };
                    let statuses = [
                        TestStatus::Success,
                        TestStatus::Failure,
                        TestStatus::Skipped,
                    ];
                    let summary: HashMap<TestStatus, usize> = {
                        use super::super::schema::test_result_in_report::dsl::*;

                        let mut summary = HashMap::new();
                        for one_status in &statuses {
                            let query = test_result_in_report
                                .select(test_id)
                                .distinct()
                                .filter(report_id.eq(&report_from_db.id))
                                .filter(status.eq(one_status.as_i32()));

                            summary.insert(
                                one_status.clone(),
                                query
                                    .load(self.0.as_ref().expect("fail to get DB"))
                                    .map(|v: Vec<String>| v.len())
                                    .unwrap_or(0),
                            );
                        }
                        summary
                    };

                    ::api::report::Report {
                        name: report_from_db.name.clone(),
                        group: report_from_db.folder.clone(),
                        created_on: report_from_db.created_on,
                        last_update: report_from_db.last_update,
                        categories: None,
                        environments,
                        summary: Some(summary),
                    }
                })
                .collect(),
        )
    }
}

pub struct GetReport {
    pub report_group: String,
    pub report_name: String,
    pub environment: Option<String>,
}
impl Message for GetReport {
    type Result = Option<::api::report::Report>;
}

impl Handler<GetReport> for super::DbReadExecutor {
    type Result = MessageResult<GetReport>;

    fn handle(&mut self, msg: GetReport, _ctx: &mut Self::Context) -> Self::Result {
        use super::super::schema::report::dsl::*;

        let report_from_db: Option<ReportDb> = report
            .filter(folder.eq(&msg.report_group))
            .filter(name.eq(&msg.report_name))
            .first(self.0.as_ref().expect("fail to get DB"))
            .ok();

        MessageResult(report_from_db.map(|report_from_db| {
            use super::super::schema::test_result_in_report::dsl::*;
            let categories: Vec<String> = test_result_in_report
                .select(category)
                .filter(report_id.eq(&report_from_db.id))
                .order(category.asc())
                .distinct()
                .load::<String>(self.0.as_ref().expect("fail to get DB"))
                .unwrap_or_else(|err| {
                    error!("error loading categories for report: {:?}", err);
                    vec![]
                });

            let mut test_results: HashMap<
                String,
                Vec<::engine::test_result::TestResult>,
            > = HashMap::new();
            categories.iter().for_each(|category_found| {
                let mut traces_query = test_result_in_report
                    .select(trace_id)
                    .filter(report_id.eq(&report_from_db.id))
                    .filter(category.eq(category_found))
                    .into_boxed();
                traces_query = match msg.environment.as_ref().map(|s| s.as_str()) {
                    Some("None") => traces_query.filter(environment.is_null()),
                    Some(v) => traces_query.filter(environment.eq(v)),
                    None => traces_query.filter(environment.is_null()),
                };
                let traces: Vec<_> = traces_query
                    .load::<String>(self.0.as_ref().expect("fail to get DB"))
                    .unwrap_or_else(|err| {
                        error!("error loading test results from category: {:?}", err);
                        vec![]
                    });
                let results = {
                    use super::super::schema::test_result::dsl::*;
                    let mut test_item_cache = super::super::helper::Cacher::new();

                    let mut tr_query = test_result.filter(trace_id.eq_any(traces)).into_boxed();
                    tr_query = match msg.environment.as_ref().map(|s| s.as_str()) {
                        Some("None") => tr_query.filter(environment.is_null()),
                        Some(v) => tr_query.filter(environment.eq(v)),
                        None => tr_query.filter(environment.is_null()),
                    };

                    tr_query
                        .order(date.desc())
                        .load::<::db::test::TestResultDb>(self.0.as_ref().expect("fail to get DB"))
                        .unwrap_or_else(|err| {
                            error!("error loading test results: {:?}", err);
                            vec![]
                        })
                        .iter()
                        .map(|tr| {
                            let test = test_item_cache
                                .get(&tr.test_id, |ti_id| {
                                    use super::super::schema::test_item::dsl::*;

                                    test_item
                                        .filter(id.eq(ti_id))
                                        .first::<::db::test::TestItemDb>(
                                            self.0.as_ref().expect("fail to get DB"),
                                        )
                                        .ok()
                                })
                                .clone();

                            let mut test_item_to_get =
                                test.clone().and_then(|t| match t.parent_id.as_ref() {
                                    "root" => None,
                                    item_id => Some(item_id.to_string()),
                                });
                            let mut path = vec![];
                            while let Some(test_item) = test_item_to_get {
                                if let Some(test) = test_item_cache
                                    .get(&test_item, |ti_id| {
                                        use super::super::schema::test_item::dsl::*;
                                        test_item
                                            .filter(id.eq(ti_id))
                                            .first::<::db::test::TestItemDb>(
                                                self.0.as_ref().expect("fail to get DB"),
                                            )
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
                                name: test
                                    .map(|t| t.name)
                                    .unwrap_or_else(|| "missing name".to_string()),
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
                            }
                        })
                        .collect::<Vec<::engine::test_result::TestResult>>()
                };
                test_results.insert(category_found.clone(), results);
            });

            let environments: Vec<String> = {
                use super::super::schema::test_result_in_report::dsl::*;
                test_result_in_report
                    .select(environment)
                    .filter(report_id.eq(&report_from_db.id))
                    .order(environment.asc())
                    .distinct()
                    .load::<Option<String>>(self.0.as_ref().expect("fail to get DB"))
                    .unwrap_or_else(|err| {
                        error!("error loading environments from report: {:?}", err);
                        vec![]
                    })
                    .iter()
                    .map(|vo| match vo {
                        Some(ref v) => v.clone(),
                        None => "None".to_string(),
                    })
                    .collect()
            };

            ::api::report::Report {
                name: report_from_db.name.clone(),
                group: report_from_db.folder.clone(),
                created_on: report_from_db.created_on,
                last_update: report_from_db.last_update,
                categories: Some(test_results),
                environments,
                summary: None,
            }
        }))
    }
}
