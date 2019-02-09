use actix::MailboxError;
use futures;

use super::query::{Column, ToGrafana, Value};

impl ToGrafana for crate::opentracing::Span {
    fn as_column_types() -> Vec<Column> {
        vec![
            Column {
                text: "Name",
                column_type: "String",
            },
            Column {
                text: "Time",
                column_type: "Date",
            },
            Column {
                text: "Duration",
                column_type: "Number",
            },
            Column {
                text: "Remote Endpoint",
                column_type: "String",
            },
        ]
    }
    fn as_columns(self) -> Vec<Value> {
        let mut column = vec![];
        column.push(Value::String(self.name.unwrap_or_else(|| "".to_string())));
        column.push(Value::Number(self.timestamp.unwrap_or(0) / 1000));
        column.push(Value::Number(self.duration.unwrap_or(0) / 1000));
        column.push(Value::String(
            self.remote_endpoint
                .and_then(|ep| ep.service_name)
                .unwrap_or_else(|| "".to_string()),
        ));
        column
    }
}
impl ToGrafana for crate::engine::test_result::TestResult {
    fn as_column_types() -> Vec<Column> {
        vec![
            Column {
                text: "Name",
                column_type: "String",
            },
            Column {
                text: "Path",
                column_type: "String",
            },
            Column {
                text: "Status",
                column_type: "String",
            },
            Column {
                text: "Environment",
                column_type: "String",
            },
            Column {
                text: "Nb Spans",
                column_type: "Number",
            },
            Column {
                text: "Time",
                column_type: "Date",
            },
            Column {
                text: "Duration",
                column_type: "Number",
            },
        ]
    }
    fn as_columns(self) -> Vec<Value> {
        let mut column = vec![];
        column.push(Value::String(self.name));
        column.push(Value::String(self.path.join("/")));
        column.push(Value::String(self.status.as_str().to_string()));
        column.push(Value::String(
            self.environment.unwrap_or_else(|| "".to_string()),
        ));
        column.push(Value::Number(i64::from(self.nb_spans)));
        column.push(Value::Number(self.date / 1000));
        column.push(Value::Number(self.duration / 1000));
        column
    }
}
impl ToGrafana for crate::api::report::Report {
    fn as_column_types() -> Vec<Column> {
        vec![
            Column {
                text: "Name",
                column_type: "String",
            },
            Column {
                text: "Last Update",
                column_type: "Date",
            },
            Column {
                text: "Success",
                column_type: "Number",
            },
            Column {
                text: "Failure",
                column_type: "Number",
            },
            Column {
                text: "Skipped",
                column_type: "Number",
            },
        ]
    }
    fn as_columns(self) -> Vec<Value> {
        let mut column = vec![];
        column.push(Value::String(self.name));
        column.push(Value::Number(self.last_update.timestamp()));
        column.push(Value::Number(
            self.summary
                .clone()
                .and_then(|summary| {
                    summary
                        .get(&crate::engine::test_result::TestStatus::Success)
                        .cloned()
                })
                .unwrap_or(0) as i64,
        ));
        column.push(Value::Number(
            self.summary
                .clone()
                .and_then(|summary| {
                    summary
                        .get(&crate::engine::test_result::TestStatus::Failure)
                        .cloned()
                })
                .unwrap_or(0) as i64,
        ));
        column.push(Value::Number(
            self.summary
                .clone()
                .and_then(|summary| {
                    summary
                        .get(&crate::engine::test_result::TestStatus::Skipped)
                        .cloned()
                })
                .unwrap_or(0) as i64,
        ));
        column
    }
}

pub enum DataQuery {
    FutureSpans(Box<futures::Future<Item = Vec<crate::opentracing::Span>, Error = MailboxError>>),
    FutureTestResults(
        Box<
            futures::Future<
                Item = Vec<crate::engine::test_result::TestResult>,
                Error = MailboxError,
            >,
        >,
    ),
    FutureReports(
        Box<futures::Future<Item = Vec<crate::api::report::Report>, Error = MailboxError>>,
    ),
}
impl futures::Future for DataQuery {
    type Item = FutureData;
    type Error = MailboxError;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self {
            DataQuery::FutureSpans(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
            DataQuery::FutureTestResults(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
            DataQuery::FutureReports(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
        }
    }
}

pub enum FutureData {
    FutureSpans(Vec<crate::opentracing::Span>),
    FutureTestResults(Vec<crate::engine::test_result::TestResult>),
    FutureReports(Vec<crate::api::report::Report>),
}
impl From<Vec<crate::opentracing::Span>> for FutureData {
    fn from(value: Vec<crate::opentracing::Span>) -> FutureData {
        FutureData::FutureSpans(value)
    }
}
impl From<Vec<crate::engine::test_result::TestResult>> for FutureData {
    fn from(value: Vec<crate::engine::test_result::TestResult>) -> FutureData {
        FutureData::FutureTestResults(value)
    }
}
impl From<Vec<crate::api::report::Report>> for FutureData {
    fn from(value: Vec<crate::api::report::Report>) -> FutureData {
        FutureData::FutureReports(value)
    }
}
