use chrono;
use actix::prelude::*;
use diesel::prelude::*;
use diesel;

use db::schema::script;
#[derive(Debug, Insertable, Queryable, Clone)]
#[table_name = "script"]
struct ScriptDb {
    id: String,
    name: String,
    source: String,
    script_type: i32,
    date_added: chrono::NaiveDateTime,
    status: i32,
}

pub struct SaveScript(pub ::engine::streams::Script);

impl ResponseType for SaveScript {
    type Item = ();
    type Error = ();
}

impl Handler<SaveScript> for super::DbExecutor {
    type Result = MessageResult<SaveScript>;

    fn handle(&mut self, msg: SaveScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        diesel::insert_into(script)
            .values(&ScriptDb {
                id: msg.0.id.unwrap().clone(),
                name: msg.0.name.clone(),
                source: msg.0.source.clone(),
                script_type: match msg.0.script_type {
                    ::engine::streams::ScriptType::Test => 0,
                    ::engine::streams::ScriptType::Span => 1,
                },
                date_added: msg.0.date_added.unwrap(),
                status: match msg.0.status.unwrap() {
                    ::engine::streams::ScriptStatus::Enabled => 0,
                    ::engine::streams::ScriptStatus::Disabled => 1,
                },
            })
            .execute(&self.0)
            .unwrap();

        Ok(())
    }
}

pub struct GetAll;

impl ResponseType for GetAll {
    type Item = Vec<::engine::streams::Script>;
    type Error = ();
}

impl Handler<GetAll> for super::DbExecutor {
    type Result = MessageResult<GetAll>;

    fn handle(&mut self, _msg: GetAll, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let scripts: Vec<ScriptDb> = script
            .order(script_type.asc())
            .order(name.asc())
            .load(&self.0)
            .expect("error loading scripts");

        Ok(scripts
            .iter()
            .map(|script_from_db| ::engine::streams::Script {
                id: Some(script_from_db.id.clone()),
                date_added: Some(script_from_db.date_added),
                script_type: match script_from_db.script_type {
                    0 => ::engine::streams::ScriptType::Test,
                    _ => ::engine::streams::ScriptType::Span,
                },
                name: script_from_db.name.clone(),
                source: script_from_db.source.clone(),
                status: Some(match script_from_db.status {
                    0 => ::engine::streams::ScriptStatus::Enabled,
                    _ => ::engine::streams::ScriptStatus::Disabled,
                }),
            })
            .collect())
    }
}

pub struct GetScript(pub String);

impl ResponseType for GetScript {
    type Item = Option<::engine::streams::Script>;
    type Error = ();
}

impl Handler<GetScript> for super::DbExecutor {
    type Result = MessageResult<GetScript>;

    fn handle(&mut self, msg: GetScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let script_found = script.filter(id.eq(msg.0)).first::<ScriptDb>(&self.0).ok();

        Ok(
            script_found.map(|script_from_db| ::engine::streams::Script {
                id: Some(script_from_db.id.clone()),
                date_added: Some(script_from_db.date_added),
                script_type: match script_from_db.script_type {
                    0 => ::engine::streams::ScriptType::Test,
                    _ => ::engine::streams::ScriptType::Span,
                },
                name: script_from_db.name.clone(),
                source: script_from_db.source.clone(),
                status: Some(match script_from_db.status {
                    0 => ::engine::streams::ScriptStatus::Enabled,
                    _ => ::engine::streams::ScriptStatus::Disabled,
                }),
            }),
        )
    }
}

pub struct DeleteScript(pub String);

impl ResponseType for DeleteScript {
    type Item = Option<::engine::streams::Script>;
    type Error = ();
}

impl Handler<DeleteScript> for super::DbExecutor {
    type Result = MessageResult<DeleteScript>;

    fn handle(&mut self, msg: DeleteScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let script_found = script
            .filter(id.eq(msg.0.clone()))
            .first::<ScriptDb>(&self.0)
            .ok();

        diesel::delete(script.filter(id.eq(msg.0)))
            .execute(&self.0)
            .expect("Error deleting script");

        Ok(
            script_found.map(|script_from_db| ::engine::streams::Script {
                id: Some(script_from_db.id.clone()),
                date_added: Some(script_from_db.date_added),
                script_type: match script_from_db.script_type {
                    0 => ::engine::streams::ScriptType::Test,
                    _ => ::engine::streams::ScriptType::Span,
                },
                name: script_from_db.name.clone(),
                source: script_from_db.source.clone(),
                status: Some(match script_from_db.status {
                    0 => ::engine::streams::ScriptStatus::Enabled,
                    _ => ::engine::streams::ScriptStatus::Disabled,
                }),
            }),
        )
    }
}
