use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;

impl ResponseType for ::engine::span::Span {
    type Item = ();
    type Error = ();
}

impl Handler<::engine::span::Span> for super::DbExecutor {
    type Result = MessageResult<::engine::span::Span>;

    fn handle(&mut self, msg: ::engine::span::Span, _: &mut Self::Context) -> Self::Result {
        use super::schema::span::dsl::*;

        diesel::insert_into(span)
            .values(&msg)
            .execute(&self.0)
            .expect("Error inserting Span");
        Ok(())
    }
}
