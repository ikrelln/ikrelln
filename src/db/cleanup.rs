use diesel;
use actix::{Handler, Message};
use diesel::prelude::*;
use chrono;

pub struct CleanUp;
impl Message for CleanUp {
    type Result = ();
}
impl Handler<CleanUp> for super::DbExecutor {
    type Result = ();

    fn handle(&mut self, _msg: CleanUp, _: &mut Self::Context) -> Self::Result {
        use super::schema::test_result::dsl::*;
        let limit = chrono::Utc::now().naive_utc()
            - chrono::Duration::seconds(::CONFIG.cleanup.delay_test_results as i64);

        let deleted = diesel::delete(
            test_result
                .filter(date.lt(limit))
                .filter(cleanup_status.eq(super::test::ResultCleanupStatus::Shell.into_i32())),
        ).execute(self.0.as_ref().unwrap())
            .ok()
            .unwrap_or(0);
        info!("deleted {} test results", deleted);
    }
}
