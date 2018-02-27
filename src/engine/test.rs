use std::str::FromStr;
use std::collections::hash_map::{Entry, HashMap};
use std::time::Duration;

use futures::{self, Future};
use actix::prelude::*;

#[cfg(feature = "python")]
use cpython::{PyDict, Python, ToPyObject};

#[derive(Debug)]
struct KnownTag {
    tag: String,
}
impl From<OpenTracingTag> for KnownTag {
    fn from(tag: OpenTracingTag) -> KnownTag {
        let tag_str: &'static str = tag.into();
        KnownTag {
            tag: format!("{}", tag_str),
        }
    }
}
impl From<IkrellnTags> for KnownTag {
    fn from(tag: IkrellnTags) -> KnownTag {
        let tag_str: &'static str = tag.into();
        KnownTag {
            tag: format!("{}", tag_str),
        }
    }
}

// OpenTracing semantics v1.1
// https://github.com/opentracing/specification/blob/master/semantic_conventions.md#span-tags-table
#[derive(Clone)]
enum OpenTracingTag {
    Component,
    DbInstance,
    DbStatement,
    DbType,
    DbUser,
    Error,
    HttpMethod,
    HttpStatusCode,
    HttpUrl,
    MessageBusDestination,
    PeerAddress,
    PeerHostname,
    PeerIpv4,
    PeerIpv6,
    PeerPort,
    PeerService,
    SamplingPriority,
    SpanKind,
}
impl From<OpenTracingTag> for &'static str {
    fn from(tag: OpenTracingTag) -> &'static str {
        match tag {
            OpenTracingTag::Component => "component",
            OpenTracingTag::DbInstance => "db.instance",
            OpenTracingTag::DbStatement => "db.statement",
            OpenTracingTag::DbType => "db.type",
            OpenTracingTag::DbUser => "db.user",
            OpenTracingTag::Error => "error",
            OpenTracingTag::HttpMethod => "http.method",
            OpenTracingTag::HttpStatusCode => "http.status_code",
            OpenTracingTag::HttpUrl => "http.url",
            OpenTracingTag::MessageBusDestination => "message_bus.destination",
            OpenTracingTag::PeerAddress => "peer.address",
            OpenTracingTag::PeerHostname => "peer.hostname",
            OpenTracingTag::PeerIpv4 => "peer.ipv4",
            OpenTracingTag::PeerIpv6 => "peer.ipv6",
            OpenTracingTag::PeerPort => "peer.port",
            OpenTracingTag::PeerService => "peer.service",
            OpenTracingTag::SamplingPriority => "sampling.priority",
            OpenTracingTag::SpanKind => "span.kind",
        }
    }
}
struct NonOpenTracingTag;
impl FromStr for OpenTracingTag {
    type Err = NonOpenTracingTag;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "component" => Ok(OpenTracingTag::Component),
            "db.instance" => Ok(OpenTracingTag::DbInstance),
            "db.statement" => Ok(OpenTracingTag::DbStatement),
            "db.type" => Ok(OpenTracingTag::DbType),
            "db.user" => Ok(OpenTracingTag::DbUser),
            "error" => Ok(OpenTracingTag::Error),
            "http.method" => Ok(OpenTracingTag::HttpMethod),
            "http.status_code" => Ok(OpenTracingTag::HttpStatusCode),
            "http.url" => Ok(OpenTracingTag::HttpUrl),
            "message_bus.destination" => Ok(OpenTracingTag::MessageBusDestination),
            "peer.address" => Ok(OpenTracingTag::PeerAddress),
            "peer.hostname" => Ok(OpenTracingTag::PeerHostname),
            "peer.ipv4" => Ok(OpenTracingTag::PeerIpv4),
            "peer.ipv6" => Ok(OpenTracingTag::PeerIpv6),
            "peer.port" => Ok(OpenTracingTag::PeerPort),
            "peer.service" => Ok(OpenTracingTag::PeerService),
            "sampling.priority" => Ok(OpenTracingTag::SamplingPriority),
            "span.kind" => Ok(OpenTracingTag::SpanKind),
            &_ => Err(NonOpenTracingTag),
        }
    }
}

#[derive(Clone)]
enum IkrellnTags {
    Class,
    Environment,
    Name,
    Result,
    StepParameters,
    StepStatus,
    StepType,
    Suite,
}
impl From<IkrellnTags> for &'static str {
    fn from(tag: IkrellnTags) -> &'static str {
        match tag {
            IkrellnTags::Class => "test.class",
            IkrellnTags::Environment => "test.environment",
            IkrellnTags::Name => "test.name",
            IkrellnTags::Result => "test.result",
            IkrellnTags::StepParameters => "test.step_parameters",
            IkrellnTags::StepStatus => "test.step_status",
            IkrellnTags::StepType => "test.step_type",
            IkrellnTags::Suite => "test.suite",
        }
    }
}
struct NonIkrellnTag;
impl FromStr for IkrellnTags {
    type Err = NonIkrellnTag;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "test.class" => Ok(IkrellnTags::Class),
            "test.environment" => Ok(IkrellnTags::Environment),
            "test.name" => Ok(IkrellnTags::Name),
            "test.result" => Ok(IkrellnTags::Result),
            "test.step_parameters" => Ok(IkrellnTags::StepParameters),
            "test.step_status" => Ok(IkrellnTags::StepStatus),
            "test.step_type" => Ok(IkrellnTags::StepType),
            "test.suite" => Ok(IkrellnTags::Suite),
            &_ => Err(NonIkrellnTag),
        }
    }
}

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
pub struct TraceDoneNow(pub String);
impl Handler<TraceDoneNow> for TraceParser {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: TraceDoneNow, ctx: &mut Context<Self>) -> Self::Result {
        ctx.notify_later(TraceDone(msg.0), Duration::new(2, 0));
        Ok(())
    }
}

#[derive(Message)]
pub struct TraceDone(pub String);
impl Handler<TraceDone> for TraceParser {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: TraceDone, ctx: &mut Context<Self>) -> Self::Result {
        let trace_parser = ::DB_EXECUTOR_POOL
            .call_fut(::db::span::GetSpans(
                ::db::span::SpanQuery::default()
                    .with_trace_id(msg.0)
                    .with_limit(1000),
            ))
            .from_err()
            .and_then(|spans| {
                if let Ok(spans) = spans {
                    //                    let mut _spans_processed: Vec<String> = vec![];
                    let te = TestResult::try_from(&spans);
                    match te {
                        Ok(te) => Ok(Some(te)),
                        Err(tag) => {
                            warn!(
                                "missing / invalid tag {:?} in trace for spans {:?}",
                                tag, spans
                            );
                            Ok(None)
                        }
                    }
                } else {
                    Ok(None)
                }
            });
        ctx.add_future(trace_parser.and_then(|test_exec| match test_exec {
            Some(test_exec) => futures::future::result(Ok(TestExecutionToSave(test_exec))),
            None => futures::future::result(Err(futures::Canceled)),
        }));

        Ok(())
    }
}

#[derive(Message, Debug)]
pub struct TestExecutionToSave(TestResult);
impl Handler<Result<TestExecutionToSave, futures::Canceled>> for TraceParser {
    type Result = Result<(), ()>;
    fn handle(
        &mut self,
        msg: Result<TestExecutionToSave, futures::Canceled>,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        if let Ok(test_execution) = msg {
            info!("got a test execution parsed: {:?}", test_execution);
            ::DB_EXECUTOR_POOL.send(test_execution.0.clone());
            actix::Arbiter::system_registry()
                .get::<::engine::streams::Streamer>()
                .send(::engine::streams::Test(test_execution.0));
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Clone)]
pub enum TestStatus {
    Success,
    Failure,
    Skipped,
}
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

#[derive(Debug, Serialize, Clone)]
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
}

#[cfg(feature = "python")]
impl ToPyObject for TestResult {
    type ObjectType = PyDict;
    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        let object = PyDict::new(py);
        object.set_item(py, "path", self.path.clone()).unwrap();
        object.set_item(py, "name", self.name.clone()).unwrap();
        object
            .set_item(py, "trace_id", self.trace_id.clone())
            .unwrap();
        object.set_item(py, "date", self.date).unwrap();
        object
            .set_item(
                py,
                "status",
                match self.status {
                    TestStatus::Success => "Success",
                    TestStatus::Failure => "Failure",
                    TestStatus::Skipped => "Skipped",
                },
            )
            .unwrap();
        object.set_item(py, "duration", self.duration).unwrap();
        if let Some(environment) = self.environment.clone() {
            object.set_item(py, "environment", environment).unwrap();
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
            .map(|v| v.to_string())
    }
    fn value_from_tag_or(
        span: &::engine::span::Span,
        tag: IkrellnTags,
        f: fn(&::engine::span::Span) -> Option<String>,
    ) -> Result<String, KnownTag> {
        match span.tags
            .get(tag.clone().into())
            .ok_or_else(|| tag.into())
            .map(|v| v.to_string())
        {
            Ok(value) => Ok(value),
            Err(err) => f(span).ok_or(err),
        }
    }

    fn try_from(spans: &Vec<::engine::span::Span>) -> Result<Self, KnownTag> {
        let main_span = spans.iter().find(|span| span.parent_id.is_none()).unwrap();
        let suite = Self::value_from_tag_or(main_span, IkrellnTags::Suite, |span| {
            span.local_endpoint.clone().and_then(|ep| ep.service_name)
        })?;
        let class = Self::value_from_tag(&main_span.tags, IkrellnTags::Class)?;

        let remote_services: Vec<String> = spans
            .iter()
            .filter_map(|span| span.clone().remote_endpoint.and_then(|ep| ep.service_name))
            .collect();
        let mut call_by_remote_endpoint = HashMap::new();
        for token in remote_services.into_iter() {
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
        })
    }
}
