use chrono;
use actix::prelude::*;
use diesel::prelude::*;
use uuid;
use diesel;

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
    category: Option<String>,
}

impl super::DbExecutor {
    fn find_report(&mut self, report_db: &ReportDb) -> Option<ReportDb> {
        use super::schema::report::dsl::*;

        report
            .filter(name.eq(report_db.name.clone()))
            .first::<ReportDb>(&self.0)
            .ok()
    }

    fn find_report_or_insert(&mut self, report_db: &ReportDb) -> String {
        use super::schema::report::dsl::*;

        match self.find_report(report_db) {
            Some(existing) => existing.id,
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
                            diesel::update(report.filter(id.eq(existing.id.clone())))
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

        let found_report_id = self.find_report_or_insert(&report);

        use super::schema::test_result_in_report::dsl::*;
        diesel::insert_into(test_result_in_report)
            .values(&TestResultInReportDb {
                test_id: msg.result.test_id.clone(),
                trace_id: msg.result.trace_id.clone(),
                report_id: found_report_id,
                category: msg.category,
            })
            .execute(&self.0)
            .ok();
    }
}
