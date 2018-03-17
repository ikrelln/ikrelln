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

#[derive(Message)]
pub struct SaveScript(pub ::engine::streams::Script);

impl Handler<SaveScript> for super::DbExecutor {
    type Result = ();

    fn handle(&mut self, msg: SaveScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        diesel::insert_into(script)
            .values(&ScriptDb {
                id: msg.0.id.unwrap().clone(),
                name: msg.0.name.clone(),
                source: msg.0.source.clone(),
                script_type: msg.0.script_type.into(),
                date_added: msg.0.date_added.unwrap(),
                status: match msg.0.status.unwrap() {
                    ::engine::streams::ScriptStatus::Enabled => 0,
                    ::engine::streams::ScriptStatus::Disabled => 1,
                },
            })
            .execute(self.0.as_ref().unwrap())
            .unwrap();
    }
}

pub struct GetAll(pub Option<Vec<::engine::streams::ScriptType>>);

impl Message for GetAll {
    type Result = Vec<::engine::streams::Script>;
}

impl Handler<GetAll> for super::DbExecutor {
    type Result = MessageResult<GetAll>;

    fn handle(&mut self, msg: GetAll, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let mut script_query = script.into_boxed();
        if let Some(types) = msg.0 {
            let types: Vec<i32> = types.iter().map(|ty| ty.clone().into()).collect();
            script_query = script_query.filter(script_type.eq_any(types))
        }
        let scripts: Vec<ScriptDb> = script_query
            .order(script_type.asc())
            .order(name.asc())
            .load(self.0.as_ref().unwrap())
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

impl Handler<GetScript> for super::DbExecutor {
    type Result = MessageResult<GetScript>;

    fn handle(&mut self, msg: GetScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let script_found = script
            .filter(id.eq(msg.0))
            .first::<ScriptDb>(self.0.as_ref().unwrap())
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

#[derive(Debug)]
pub struct DeleteScript(pub String);

impl Message for DeleteScript {
    type Result = Option<::engine::streams::Script>;
}

impl Handler<DeleteScript> for super::DbExecutor {
    type Result = MessageResult<DeleteScript>;

    fn handle(&mut self, msg: DeleteScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        let script_found = script
            .filter(id.eq(&msg.0))
            .first::<ScriptDb>(self.0.as_ref().unwrap())
            .ok();

        diesel::delete(script.filter(id.eq(msg.0)))
            .execute(self.0.as_ref().unwrap())
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

#[derive(Message)]
pub struct UpdateScript(pub ::engine::streams::Script);

impl Handler<UpdateScript> for super::DbExecutor {
    type Result = ();

    fn handle(&mut self, msg: UpdateScript, _: &mut Self::Context) -> Self::Result {
        use super::schema::script::dsl::*;
        diesel::update(script.filter(id.eq(&msg.0.id.unwrap())))
            .set((
                name.eq(&msg.0.name),
                source.eq(&msg.0.source),
                status.eq(match msg.0.status.unwrap() {
                    ::engine::streams::ScriptStatus::Enabled => 0,
                    ::engine::streams::ScriptStatus::Disabled => 1,
                }),
            ))
            .execute(self.0.as_ref().unwrap())
            .unwrap();
    }
}
