use actix::prelude::*;
use actix::registry::SystemService;
use futures::future::*;

#[cfg(feature = "python")]
use cpython::{FromPyObject, PyDict, PyObject, PyResult, Python};
use chrono;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ScriptType {
    StreamSpan,

    // Python function that can act on a test
    // def on_test(test):
    //     import requests
    //     import json
    //     requests.post("https://requestb.in/XXXXXXX", data=json.dumps(test))
    StreamTest,

    // Python function that can act on a test
    // def reports_for_test(test):
    //     return [{'group': 'TestClass', ''name': test.tags['test.class'], 'category': None}]
    ReportFilterTestResult,

    // JS script that returns HTML that will be displayed on each test in test detail view
    // (test) => '<a href="http://google.com">' + test.name + '</a>'
    UITest,

    // JS script that returns HTML that will be displayed on each test result in test detail view
    // (result, spans) => '<a href="http://spans.com">' + spans.length + ' spans</a>'
    UITestResult,
}
impl From<i32> for ScriptType {
    fn from(val: i32) -> ScriptType {
        match val {
            0 => ScriptType::StreamSpan,
            1 => ScriptType::StreamTest,
            2 => ScriptType::UITest,
            3 => ScriptType::UITestResult,
            4 => ScriptType::ReportFilterTestResult,
            _ => ScriptType::StreamTest,
        }
    }
}
impl Into<i32> for ScriptType {
    fn into(self) -> i32 {
        match self {
            ScriptType::StreamSpan => 0,
            ScriptType::StreamTest => 1,
            ScriptType::UITest => 2,
            ScriptType::UITestResult => 3,
            ScriptType::ReportFilterTestResult => 4,
        }
    }
}
impl Into<String> for ScriptType {
    fn into(self) -> String {
        match self {
            ScriptType::StreamSpan => "StreamSpan".to_string(),
            ScriptType::StreamTest => "StreamTest".to_string(),
            ScriptType::UITest => "UITest".to_string(),
            ScriptType::UITestResult => "UITestResult".to_string(),
            ScriptType::ReportFilterTestResult => "ReportFilterTestResult".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ScriptStatus {
    Enabled,
    Disabled,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
    type Result = ();

    fn handle(&mut self, _msg: LoadScripts, _ctx: &mut Context<Self>) -> Self::Result {
        Arbiter::handle().spawn_fn(move || {
            ::DB_EXECUTOR_POOL
                .send(::db::scripts::GetAll(Some(vec![
                    ScriptType::StreamTest,
                    ScriptType::ReportFilterTestResult,
                ])))
                .then(|scripts| {
                    if let Ok(scripts) = scripts {
                        actix::Arbiter::system_registry()
                            .get::<::engine::streams::Streamer>()
                            .do_send(UpdateScripts(scripts));
                    }
                    result(Ok(()))
                })
        })
    }
}

#[derive(Message)]
pub struct UpdateScripts(Vec<Script>);

impl Handler<UpdateScripts> for Streamer {
    type Result = ();

    fn handle(&mut self, msg: UpdateScripts, _ctx: &mut Context<Self>) -> Self::Result {
        self.scripts = msg.0;
    }
}

#[derive(Message)]
pub struct AddScript(pub Script);
impl Handler<AddScript> for Streamer {
    type Result = ();

    fn handle(&mut self, msg: AddScript, _ctx: &mut Context<Self>) -> Self::Result {
        self.scripts.push(msg.0);
    }
}

#[derive(Message)]
pub struct RemoveScript(pub Script);
impl Handler<RemoveScript> for Streamer {
    type Result = ();

    fn handle(&mut self, msg: RemoveScript, _ctx: &mut Context<Self>) -> Self::Result {
        let index = self.scripts
            .iter()
            .position(|x| (*x.id.clone().unwrap()) == msg.0.id.clone().unwrap())
            .unwrap();
        self.scripts.remove(index);
    }
}

#[cfg(feature = "python")]
#[derive(Debug)]
struct ReportTarget {
    group: String,
    name: String,
    category: Option<String>,
}
#[cfg(feature = "python")]
impl<'a> FromPyObject<'a> for ReportTarget {
    fn extract(py: Python, obj: &'a PyObject) -> PyResult<Self> {
        let locals = PyDict::new(py);
        locals.set_item(py, "obj", obj).unwrap();

        Ok(ReportTarget {
            group: py.eval("obj['group']", None, Some(&locals))?.extract(py)?,
            name: py.eval("obj['name']", None, Some(&locals))?.extract(py)?,
            category: py.eval("obj['category']", None, Some(&locals))?
                .extract(py)?,
        })
    }
}

#[derive(Message, Debug)]
pub struct Test(pub ::engine::test::TestResult);
impl Handler<Test> for Streamer {
    type Result = ();

    #[cfg(feature = "python")]
    fn handle(&mut self, msg: Test, _ctx: &mut Context<Self>) -> Self::Result {
        if self.scripts.len() > 0 {
            let gil = Python::acquire_gil();
            let py = gil.python();

            let locals = PyDict::new(py);
            locals.set_item(py, "test", msg.0.clone()).unwrap();

            let stream_test_script: Vec<&Script> = self.scripts
                .iter()
                .filter(|script| match script.script_type {
                    ScriptType::StreamTest => true,
                    _ => false,
                })
                .collect();
            for script in stream_test_script {
                match py.run(script.source.as_ref(), None, Some(&locals)) {
                    Ok(_) => match py.eval("on_test(test)", None, Some(&locals)) {
                        Ok(_) => (),
                        Err(err) => warn!(
                            "error executing python script {}: {:?}",
                            script.id.clone().unwrap(),
                            err
                        ), // TODO disable script after failure
                    },
                    _ => (),
                }
            }

            let report_filter_test_script: Vec<&Script> = self.scripts
                .iter()
                .filter(|script| match script.script_type {
                    ScriptType::ReportFilterTestResult => true,
                    _ => false,
                })
                .collect();
            for script in report_filter_test_script {
                match py.run(script.source.as_ref(), None, Some(&locals)) {
                    Ok(_) => match py.eval("reports_for_test(test)", None, Some(&locals)) {
                        Ok(py_reports) => {
                            let reports = py_reports.extract::<Vec<ReportTarget>>(py);
                            if let Ok(reports) = reports {
                                for report in reports {
                                    actix::Arbiter::system_registry()
                                        .get::<::engine::report::Reporter>()
                                        .do_send(::engine::report::ResultForReport {
                                            report_group: report.group,
                                            report_name: report.name,
                                            category: report.category,
                                            result: msg.0.clone(),
                                        })
                                }
                            }
                        }
                        Err(err) => warn!(
                            "error executing python script {}: {:?}",
                            script.id.clone().unwrap(),
                            err
                        ), // TODO disable script after failure
                    },
                    _ => (),
                }
            }
        }
    }

    #[cfg(not(feature = "python"))]
    fn handle(&mut self, _msg: Test, _ctx: &mut Context<Self>) -> Self::Result {}
}
