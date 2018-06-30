use actix::prelude::*;
use chrono;
// use diesel;
use diesel::prelude::*;

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

pub struct GetAll(pub Option<Vec<::engine::streams::ScriptType>>);

impl Message for GetAll {
    type Result = Vec<::engine::streams::Script>;
}

impl Handler<GetAll> for super::DbReadExecutor {
    type Result = MessageResult<GetAll>;

    fn handle(&mut self, msg: GetAll, _: &mut Self::Context) -> Self::Result {
        use super::super::schema::script::dsl::*;
        let mut script_query = script.into_boxed();
        if let Some(types) = msg.0 {
            let types: Vec<i32> = types.iter().map(|ty| ty.clone().into()).collect();
            script_query = script_query.filter(script_type.eq_any(types))
        }
        let scripts: Vec<ScriptDb> = script_query
            .order(script_type.asc())
            .order(name.asc())
            .load(self.0.as_ref().expect("fail to get DB"))
            .unwrap_or_else(|err| {
                error!("error loading scripts: {:?}", err);
                vec![]
            });

        MessageResult(
            scripts
                .iter()
                .map(|script_from_db| ::engine::streams::Script {
                    id: Some(script_from_db.id.clone()),
                    date_added: Some(script_from_db.date_added),
                    script_type: script_from_db.script_type.into(),
                    name: script_from_db.name.clone(),
                    source: script_from_db.source.clone(),
                    status: Some(match script_from_db.status {
                        0 => ::engine::streams::ScriptStatus::Enabled,
                        _ => ::engine::streams::ScriptStatus::Disabled,
                    }),
                })
                .collect(),
        )
    }
}

pub struct GetScript(pub String);

impl Message for GetScript {
    type Result = Option<::engine::streams::Script>;
}

impl Handler<GetScript> for super::DbReadExecutor {
    type Result = MessageResult<GetScript>;

    fn handle(&mut self, msg: GetScript, _: &mut Self::Context) -> Self::Result {
        use super::super::schema::script::dsl::*;
        let script_found = script
            .filter(id.eq(msg.0))
            .first::<ScriptDb>(self.0.as_ref().expect("fail to get DB"))
            .ok();

        MessageResult(
            script_found.map(|script_from_db| ::engine::streams::Script {
                id: Some(script_from_db.id.clone()),
                date_added: Some(script_from_db.date_added),
                script_type: script_from_db.script_type.into(),
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
