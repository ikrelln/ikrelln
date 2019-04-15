use std::collections::hash_map::{Entry, HashMap};

use actix::prelude::*;
use futures::{future, Future};

#[cfg(feature = "python")]
use cpython::{PyDict, Python, ToPyObject};

use crate::opentracing::tags::{IkrellnTags, KnownTag, OpenTracingTag};

#[derive(Default)]
pub struct TraceParser;
impl Actor for TraceParser {
    type Context = Context<Self>;
}
impl actix::Supervised for TraceParser {}

impl actix::SystemService for TraceParser {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

#[derive(Message)]
pub struct TraceDone(pub String);
impl Handler<TraceDone> for TraceParser {
    type Result = ();

    fn handle(&mut self, msg: TraceDone, _ctx: &mut Context<Self>) -> Self::Result {
        Arbiter::spawn(
            crate::DB_READ_EXECUTOR_POOL
                .send(crate::db::read::span::GetSpans(
                    crate::db::read::span::SpanQuery::default()
                        .with_trace_id(msg.0)
                        .with_limit(1000),
                ))
                .map(|spans| {
                    let te = TestResult::try_from(&spans);
                    match te {
                        Ok(te) => Some(te),
                        Err(tag) => {
                            warn!(
                                "missing / invalid tag {:?} in trace for spans {:?}",
                                tag, spans
                            );
                            None
                        }
                    }
                })
                .then(|test_exec| {
                    if let Ok(Some(test_exec)) = test_exec {
                        super::test_result::TraceParser::from_registry()
                            .do_send(TestExecutionToSave(test_exec));
                    }
                    future::result(Ok(()))
                }),
        )
    }
}

#[derive(Message, Debug)]
pub struct TestExecutionToSave(TestResult);

impl Handler<TestExecutionToSave> for TraceParser {
    type Result = ();

    fn handle(&mut self, msg: TestExecutionToSave, _ctx: &mut Context<Self>) -> Self::Result {
        Arbiter::spawn(
            crate::DB_EXECUTOR_POOL
                .send(msg.0.clone())
                .then(|test_result| {
                    if let Ok(test_result) = test_result {
                        crate::engine::streams::Streamer::from_registry()
                            .do_send(crate::engine::streams::Test(test_result.clone()));
                        crate::engine::report::Reporter::from_registry()
                            .do_send(crate::engine::report::ComputeReportsForResult(test_result));
                    }
                    future::result(Ok(()))
                }),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Hash)]
pub enum TestStatus {
    Success,
    Failure,
    Skipped,
    Any,
}
impl Eq for TestStatus {}
impl TestStatus {
    fn try_from(s: &str) -> Result<Self, KnownTag> {
        match s.to_lowercase().as_ref() {
            "success" => Ok(TestStatus::Success),
            "failure" => Ok(TestStatus::Failure),
            "skipped" => Ok(TestStatus::Skipped),
            _ => Err(IkrellnTags::Result.into()),
        }
    }
}
impl From<i32> for TestStatus {
    fn from(v: i32) -> Self {
        match v {
            0 => crate::engine::test_result::TestStatus::Success,
            1 => crate::engine::test_result::TestStatus::Failure,
            2 => crate::engine::test_result::TestStatus::Skipped,
            _ => crate::engine::test_result::TestStatus::Failure,
        }
    }
}
impl TestStatus {
    pub fn as_i32(&self) -> i32 {
        match self {
            crate::engine::test_result::TestStatus::Success => 0,
            crate::engine::test_result::TestStatus::Failure => 1,
            crate::engine::test_result::TestStatus::Skipped => 2,
            crate::engine::test_result::TestStatus::Any => 3,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            crate::engine::test_result::TestStatus::Success => "Success",
            crate::engine::test_result::TestStatus::Failure => "Failure",
            crate::engine::test_result::TestStatus::Skipped => "Skipped",
            crate::engine::test_result::TestStatus::Any => "Any",
        }
    }
}
impl Into<i32> for TestStatus {
    fn into(self) -> i32 {
        self.as_i32()
    }
}
impl Into<&'static str> for TestStatus {
    fn into(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestResult {
    pub test_id: String,
    pub path: Vec<String>,
    pub name: String,
    pub trace_id: String,
    pub date: i64,
    pub status: TestStatus,
    pub duration: i64,
    pub environment: Option<String>,
    pub components_called: HashMap<String, i32>,
    pub nb_spans: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub main_span: Option<crate::opentracing::Span>,
}

#[cfg(feature = "python")]
impl ToPyObject for TestResult {
    type ObjectType = PyDict;
    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        let object = PyDict::new(py);
        object
            .set_item(py, "test_id", self.test_id.clone())
            .unwrap();
        object.set_item(py, "path", self.path.clone()).unwrap();
        object.set_item(py, "name", self.name.clone()).unwrap();
        object
            .set_item(py, "trace_id", self.trace_id.clone())
            .unwrap();
        object.set_item(py, "date", self.date).unwrap();
        object.set_item(py, "status", self.status.as_str()).unwrap();
        object.set_item(py, "duration", self.duration).unwrap();
        if let Some(environment) = self.environment.clone() {
            object.set_item(py, "environment", environment).unwrap();
        }
        if let Some(main_span) = self.main_span.clone() {
            object.set_item(py, "main_span", main_span).unwrap();
        }
        object
    }
}

impl TestResult {
    fn value_from_tag<T>(tags: &HashMap<String, String>, tag: T) -> Result<String, KnownTag>
    where
        T: Clone,
        KnownTag: From<T>,
        &'static str: From<T>,
    {
        tags.get(tag.clone().into())
            .ok_or_else(|| tag.into())
            .map(std::string::ToString::to_string)
    }
    fn value_from_tag_or(
        span: &crate::opentracing::Span,
        tag: IkrellnTags,
        f: fn(&crate::opentracing::Span) -> Option<String>,
    ) -> Result<String, KnownTag> {
        match span
            .tags
            .get(tag.clone().into())
            .ok_or_else(|| tag.into())
            .map(std::string::ToString::to_string)
        {
            Ok(value) => Ok(value),
            Err(err) => f(span).ok_or(err),
        }
    }

    fn try_from(spans: &[crate::opentracing::Span]) -> Result<Self, KnownTag> {
        let main_span = match spans.iter().find(|span| span.parent_id.is_none()) {
            Some(span) => span,
            None => return Err(IkrellnTags::StepType.into()),
        };
        let suite = Self::value_from_tag_or(main_span, IkrellnTags::Suite, |span| {
            span.local_endpoint.clone().and_then(|ep| ep.service_name)
        })?;
        let class = Self::value_from_tag(&main_span.tags, IkrellnTags::Class)?;

        let remote_services: Vec<String> = spans
            .iter()
            .filter_map(|span| span.clone().remote_endpoint.and_then(|ep| ep.service_name))
            .collect();
        let mut call_by_remote_endpoint = HashMap::new();
        for token in remote_services {
            let item = call_by_remote_endpoint.entry(token);
            match item {
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry.get() + 1;
                }
                Entry::Vacant(entry) => {
                    entry.insert(1);
                }
            }
        }

        Ok(TestResult {
            test_id: "n/a".to_string(),
            path: vec![suite, class],
            name: Self::value_from_tag_or(main_span, IkrellnTags::Name, |span| span.name.clone())?,
            trace_id: main_span.trace_id.clone(),
            date: main_span.timestamp.ok_or(KnownTag {
                tag: "ts".to_string(),
            })?,
            status: TestStatus::try_from(&Self::value_from_tag_or(
                main_span,
                IkrellnTags::Result,
                |span| {
                    Self::value_from_tag(&span.tags, OpenTracingTag::Error)
                        .ok()
                        .map(|v| match v.to_lowercase().as_ref() {
                            "true" => "failure".to_string(),
                            other => other.to_string(),
                        })
                },
            )?)?,
            duration: main_span.duration.ok_or(KnownTag {
                tag: "duration".to_string(),
            })?,
            environment: Self::value_from_tag(&main_span.tags, IkrellnTags::Environment).ok(),
            components_called: call_by_remote_endpoint,
            nb_spans: spans.len() as i32,
            main_span: Some(main_span.clone()),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use uuid;

    use crate::opentracing::span::Kind;
    use crate::opentracing::tags::IkrellnTags;
    use crate::opentracing::Span;

    use super::*;

    #[test]
    fn can_get_test_result_from_span() {
        let trace_id = uuid::Uuid::new_v4().to_string();

        let mut tags: HashMap<String, String> = HashMap::new();
        tags.insert(
            String::from({
                let tag: &str = IkrellnTags::Suite.into();
                tag
            }),
            "test_suite".to_string(),
        );
        tags.insert(
            String::from({
                let tag: &str = IkrellnTags::Class.into();
                tag
            }),
            "test_class".to_string(),
        );
        tags.insert(
            String::from({
                let tag: &str = IkrellnTags::Result.into();
                tag
            }),
            "success".to_string(),
        );

        let spans = vec![Span {
            trace_id: trace_id.to_string(),
            id: trace_id.clone(),
            parent_id: None,
            name: Some("span_name".to_string()),
            kind: Some(Kind::CLIENT),
            duration: Some(25),
            timestamp: Some(50),
            debug: false,
            shared: false,
            local_endpoint: None,
            remote_endpoint: None,
            annotations: vec![],
            tags,
            binary_annotations: vec![],
        }];

        let tr = TestResult::try_from(&spans);
        assert!(tr.is_ok());
    }

}
