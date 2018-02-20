use actix::prelude::*;
use actix::SystemService;
use futures;
use futures::future::*;

#[cfg(feature = "python")]
use cpython::{PyDict, Python};
use chrono;

#[derive(Serialize, Deserialize, Clone)]
pub enum ScriptType {
    Span,
    Test,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ScriptStatus {
    Enabled,
    Disabled,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Script {
    pub id: Option<String>,
    pub name: String,
    pub source: String,
    pub script_type: ScriptType,
    pub date_added: Option<chrono::NaiveDateTime>,
    pub status: Option<ScriptStatus>,
}

pub struct Streamer {
    scripts: Vec<Script>,
}

impl Actor for Streamer {
    type Context = Context<Self>;
}

impl Default for Streamer {
    fn default() -> Self {
        Streamer { scripts: vec![] }
    }
}

impl Supervised for Streamer {}
impl SystemService for Streamer {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {
        info!("started Streamer")
    }
}

#[derive(Message)]
pub struct LoadScripts;
impl Handler<LoadScripts> for Streamer {
    type Result = Result<(), ()>;

    fn handle(&mut self, _msg: LoadScripts, ctx: &mut Context<Self>) -> Self::Result {
        let loaded = ::DB_EXECUTOR_POOL.call_fut(::db::scripts::GetAll);
        ctx.add_future(loaded.and_then(|scripts| match scripts {
            Ok(scripts) => futures::future::result(Ok(UpdateScripts(scripts))),
            _ => futures::future::result(Err(futures::Canceled)),
        }));
        Ok(())
    }
}

#[derive(Message)]
pub struct UpdateScripts(Vec<Script>);
impl Handler<Result<UpdateScripts, futures::Canceled>> for Streamer {
    type Result = Result<(), ()>;
    fn handle(
        &mut self,
        msg: Result<UpdateScripts, futures::Canceled>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        if let Ok(scripts) = msg {
            self.scripts = scripts.0;
        }
        Ok(())
    }
}

#[derive(Message)]
pub struct AddScript(pub Script);
impl Handler<AddScript> for Streamer {
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: AddScript, _ctx: &mut Context<Self>) -> Self::Result {
        self.scripts.push(msg.0);
        Ok(())
    }
}

#[derive(Message)]
pub struct RemoveScript(pub Script);
impl Handler<RemoveScript> for Streamer {
    type Result = Result<(), ()>;
    fn handle(&mut self, msg: RemoveScript, _ctx: &mut Context<Self>) -> Self::Result {
        let index = self.scripts
            .iter()
            .position(|x| (*x.id.clone().unwrap()) == msg.0.id.clone().unwrap())
            .unwrap();
        self.scripts.remove(index);
        Ok(())
    }
}

#[derive(Message, Debug)]
pub struct Test(pub ::engine::test::TestResult);
impl Handler<Test> for Streamer {
    type Result = Result<(), ()>;

    #[cfg(feature = "python")]
    fn handle(&mut self, msg: Test, _ctx: &mut Context<Self>) -> Self::Result {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let locals = PyDict::new(py);
        locals.set_item(py, "test", msg.0).unwrap();

        for script in self.scripts.clone() {
            py.run(script.source.as_ref(), None, Some(&locals)).unwrap();
        }
        Ok(())
    }

    #[cfg(not(feature = "python"))]
    fn handle(&mut self, _msg: Test, _ctx: &mut Context<Self>) -> Self::Result {
        Ok(())
    }
}
