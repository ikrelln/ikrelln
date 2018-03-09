use std::collections::HashMap;

use chrono;
use actix::prelude::*;
use diesel::prelude::*;
use uuid;
use diesel;
use serde_json;

use db::schema::report;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "report"]
struct ReportDb {
    id: String,
    name: String,
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
}

impl super::DbExecutor {
    fn find_report(&mut self, report_db: &ReportDb) -> Option<ReportDb> {
        use super::schema::report::dsl::*;

        report
            .filter(name.eq(&report_db.name))
            .first::<ReportDb>(&self.0)
            .ok()
    }

    fn update_report_or_create(&mut self, report_db: &ReportDb) -> String {
        use super::schema::report::dsl::*;

        match self.find_report(report_db) {
            Some(existing) => {
                diesel::update(report.filter(id.eq(&existing.id)))
                    .set(last_update.eq(report_db.last_update))
                    .execute(&self.0)
                    .expect("error updating report last update date");
                existing.id
            }
            None => {
                let new_id = uuid::Uuid::new_v4().hyphenated().to_string();
                let could_insert = diesel::insert_into(report)
                    .values(&ReportDb {
                        id: new_id.clone(),
                        ..(*report_db).clone()
                    })
                    .execute(&self.0);
                if could_insert.is_err() {
                    self.find_report(report_db)
                        .map(|existing| {
                            diesel::update(report.filter(id.eq(&existing.id)))
                                .set(last_update.eq(report_db.last_update))
                                .execute(&self.0)
                                .expect("error updating report last update date");
                            existing.id
                        })
                        .unwrap()
                } else {
                    new_id
                }
            }
        }
    }
}

impl Handler<::engine::report::ResultForReport> for super::DbExecutor {
    type Result = ();

    fn handle(
        &mut self,
        msg: ::engine::report::ResultForReport,
        _: &mut Self::Context,
    ) -> Self::Result {
        let report = ReportDb {
            id: "n/a".to_string(),
            name: msg.report_name.clone(),
            created_on: chrono::Utc::now().naive_utc(),
            last_update: chrono::Utc::now().naive_utc(),
        };

        let found_report_id = self.update_report_or_create(&report);

        use super::schema::test_result_in_report::dsl::*;
        let mut find_tr = test_result_in_report
            .filter(report_id.eq(&found_report_id))
            .filter(test_id.eq(&msg.result.test_id))
            .into_boxed();
        if let Some(category_from_input) = msg.category.clone() {
            find_tr = find_tr.filter(category.eq(category_from_input));
        } else {
            find_tr = find_tr.filter(category.eq(&msg.report_name));
        }
        if let Some(environment_from_input) = msg.result.environment.clone() {
            find_tr = find_tr.filter(environment.eq(environment_from_input));
        } else {
            find_tr = find_tr.filter(environment.is_null());
        }
        if find_tr
            .first::<TestResultInReportDb>(&self.0)
            .ok()
            .is_some()
        {
            match msg.category.clone() {
                Some(category_from_input) => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(category_from_input))
                            .filter(environment.eq(msg.result.environment)),
                    ).set(trace_id.eq(msg.result.trace_id))
                        .execute(&self.0)
                        .ok();
                }
                None => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(&msg.report_name))
                            .filter(environment.eq(msg.result.environment)),
                    ).set(trace_id.eq(msg.result.trace_id))
                        .execute(&self.0)
                        .ok();
                }
            };
        } else {
            diesel::insert_into(test_result_in_report)
                .values(&TestResultInReportDb {
                    test_id: msg.result.test_id.clone(),
                    trace_id: msg.result.trace_id,
                    report_id: found_report_id.clone(),
                    category: msg.category.unwrap_or(msg.report_name.clone()),
                    environment: msg.result.environment,
                })
                .execute(&self.0)
                .ok();
        }
    }
}

pub struct GetAll;
impl Message for GetAll {
    type Result = Vec<::api::report::Report>;
}

impl Handler<GetAll> for super::DbExecutor {
    type Result = MessageResult<GetAll>;

    fn handle(&mut self, _msg: GetAll, _ctx: &mut Self::Context) -> Self::Result {
        use super::schema::report::dsl::*;

        let reports: Vec<ReportDb> = report
            .order(last_update.desc())
            .load(&self.0)
            .expect("error loading reports");

        MessageResult(
            reports
                .iter()
                .map(|report_from_db| {
                    let environments: Vec<String> = {
                        use super::schema::test_result_in_report::dsl::*;
                        test_result_in_report
                            .select(environment)
                            .filter(report_id.eq(&report_from_db.id))
                            .order(environment.asc())
                            .distinct()
                            .load::<Option<String>>(&self.0)
                            .expect("can load environments from test results")
                            .iter()
                            .map(|vo| match vo {
                                &Some(ref v) => v.clone(),
                                &None => "None".to_string(),
                            })
                            .collect()
                    };

                    ::api::report::Report {
                        name: report_from_db.name.clone(),
                        created_on: report_from_db.created_on,
                        last_update: report_from_db.last_update,
                        categories: None,
                        environments: environments,
                    }
                })
                .collect(),
        )
    }
}

pub struct GetReport {
    pub report_name: String,
    pub environment: Option<String>,
}
impl Message for GetReport {
    type Result = Option<::api::report::Report>;
}

impl Handler<GetReport> for super::DbExecutor {
    type Result = MessageResult<GetReport>;

    fn handle(&mut self, msg: GetReport, _ctx: &mut Self::Context) -> Self::Result {
        use super::schema::report::dsl::*;

        let report_from_db: Option<ReportDb> =
            report.filter(name.eq(&msg.report_name)).first(&self.0).ok();

        MessageResult(report_from_db.map(|report_from_db| {
            use super::schema::test_result_in_report::dsl::*;
            let categories: Vec<String> = test_result_in_report
                .select(category)
                .filter(report_id.eq(&report_from_db.id))
                .order(category.asc())
                .distinct()
                .load::<String>(&self.0)
                .expect("can load categories from test results");

            let mut test_results: HashMap<String, Vec<::engine::test::TestResult>> = HashMap::new();
            categories.iter().for_each(|category_found| {
                let traces: Vec<_> = test_result_in_report
                    .select(trace_id)
                    .filter(report_id.eq(&report_from_db.id))
                    .filter(category.eq(category_found))
                    .load::<String>(&self.0)
                    .expect("can load test results");
                let results = {
                    use super::schema::test_result::dsl::*;
                    let mut test_item_cache = super::helper::Cacher::new();

                    let mut tr_query = test_result.filter(trace_id.eq_any(traces)).into_boxed();
                    tr_query = match msg.environment.as_ref().map(|s| s.as_str()) {
                        Some("None") => tr_query.filter(environment.is_null()),
                        Some(v) => tr_query.filter(environment.eq(v)),
                        None => tr_query.filter(environment.is_null()),
                    };

                    tr_query
                        .order(date.desc())
                        .load::<::db::test::TestResultDb>(&self.0)
                        .expect("can load test results")
                        .iter()
                        .map(|tr| {
                            let test = test_item_cache
                                .get(&tr.test_id, |ti_id| {
                                    use super::schema::test_item::dsl::*;

                                    test_item
                                        .filter(id.eq(ti_id))
                                        .first::<::db::test::TestItemDb>(&self.0)
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
                                            .first::<::db::test::TestItemDb>(&self.0)
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
                                path,
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
                                components_called: serde_json::from_str(&tr.components_called)
                                    .unwrap(),
                                nb_spans: tr.nb_spans,
                                main_span: None,
                            }
                        })
                        .collect::<Vec<::engine::test::TestResult>>()
                };
                test_results.insert(category_found.clone(), results);
            });

            let environments: Vec<String> = {
                use super::schema::test_result_in_report::dsl::*;
                test_result_in_report
                    .select(environment)
                    .filter(report_id.eq(&report_from_db.id))
                    .order(environment.asc())
                    .distinct()
                    .load::<Option<String>>(&self.0)
                    .expect("can load environments from test results")
                    .iter()
                    .map(|vo| match vo {
                        &Some(ref v) => v.clone(),
                        &None => "None".to_string(),
                    })
                    .collect()
            };

            ::api::report::Report {
                name: report_from_db.name.clone(),
                created_on: report_from_db.created_on,
                last_update: report_from_db.last_update,
                categories: Some(test_results),
                environments: environments,
            }
        }))
    }
}
