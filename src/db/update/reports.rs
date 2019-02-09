use actix::prelude::*;
use chrono;
use diesel;
use diesel::prelude::*;
use uuid;

use crate::db::schema::report;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "report"]
pub struct ReportDb {
    pub id: String,
    name: String,
    folder: String,
    created_on: chrono::NaiveDateTime,
    last_update: chrono::NaiveDateTime,
}

use crate::db::schema::test_result_in_report;
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

impl super::DbExecutor {
    fn find_report(&mut self, report_db: &ReportDb) -> Option<ReportDb> {
        use super::super::schema::report::dsl::*;
        report
            .filter(folder.eq(&report_db.folder))
            .filter(name.eq(&report_db.name))
            .first::<ReportDb>(self.0.as_ref().expect("fail to get DB"))
            .ok()
    }

    fn update_report_or_create(&mut self, report_db: &ReportDb) -> String {
        use super::super::schema::report::dsl::*;

        match self.find_report(report_db) {
            Some(existing) => {
                diesel::update(report.filter(id.eq(&existing.id)))
                    .set(last_update.eq(report_db.last_update))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
                existing.id
            }
            None => {
                let new_id = uuid::Uuid::new_v4().to_hyphenated().to_string();
                let could_insert = diesel::insert_into(report)
                    .values(&ReportDb {
                        id: new_id.clone(),
                        ..(*report_db).clone()
                    })
                    .execute(self.0.as_ref().expect("fail to get DB"));
                if could_insert.is_err() {
                    self.find_report(report_db)
                        .map(|existing| {
                            diesel::update(report.filter(id.eq(&existing.id)))
                                .set(last_update.eq(report_db.last_update))
                                .execute(self.0.as_ref().expect("fail to get DB"))
                                .ok();
                            existing.id
                        })
                        .expect("fail to find report")
                } else {
                    new_id
                }
            }
        }
    }
}

impl Handler<crate::engine::report::ResultForReport> for super::DbExecutor {
    type Result = ();

    fn handle(
        &mut self,
        msg: crate::engine::report::ResultForReport,
        _: &mut Self::Context,
    ) -> Self::Result {
        let report = ReportDb {
            id: "n/a".to_string(),
            name: msg.report_name.clone(),
            folder: msg.report_group.clone(),
            created_on: chrono::Utc::now().naive_utc(),
            last_update: chrono::Utc::now().naive_utc(),
        };

        let found_report_id = self.update_report_or_create(&report);

        use super::super::schema::test_result_in_report::dsl::*;
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
            .first::<TestResultInReportDb>(self.0.as_ref().expect("fail to get DB"))
            .ok()
            .is_some()
        {
            match (msg.category, msg.result.environment) {
                (Some(category_from_input), Some(environment_from_input)) => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(category_from_input))
                            .filter(environment.eq(environment_from_input)),
                    )
                    .set((
                        trace_id.eq(msg.result.trace_id),
                        status.eq(msg.result.status.as_i32()),
                    ))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
                }
                (Some(category_from_input), None) => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(category_from_input))
                            .filter(environment.is_null()),
                    )
                    .set((
                        trace_id.eq(msg.result.trace_id),
                        status.eq(msg.result.status.as_i32()),
                    ))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
                }

                (None, Some(environment_from_input)) => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(&msg.report_name))
                            .filter(environment.eq(environment_from_input)),
                    )
                    .set((
                        trace_id.eq(msg.result.trace_id),
                        status.eq(msg.result.status.as_i32()),
                    ))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
                }
                (None, None) => {
                    diesel::update(
                        test_result_in_report
                            .filter(report_id.eq(&found_report_id))
                            .filter(test_id.eq(&msg.result.test_id))
                            .filter(category.eq(&msg.report_name))
                            .filter(environment.is_null()),
                    )
                    .set((
                        trace_id.eq(msg.result.trace_id),
                        status.eq(msg.result.status.as_i32()),
                    ))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
                }
            };
        } else {
            diesel::insert_into(test_result_in_report)
                .values(&TestResultInReportDb {
                    test_id: msg.result.test_id.clone(),
                    trace_id: msg.result.trace_id.clone(),
                    report_id: found_report_id.clone(),
                    category: msg
                        .category
                        .clone()
                        .unwrap_or_else(|| msg.report_name.clone()),
                    environment: msg.result.environment,
                    status: msg.result.status.into(),
                })
                .execute(self.0.as_ref().expect("fail to get DB"))
                .ok();
        }
    }
}
