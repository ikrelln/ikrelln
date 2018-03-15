use futures;
use actix::MailboxError;

use super::query::{Column, ToGrafana, Value};

impl ToGrafana for ::opentracing::Span {
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
impl ToGrafana for ::engine::test_result::TestResult {
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
        column.push(Value::String(self.status.into_str().to_string()));
        column.push(Value::String(
            self.environment.unwrap_or_else(|| "".to_string()),
        ));
        column.push(Value::Number(self.nb_spans as i64));
        column.push(Value::Number(self.date / 1000));
        column.push(Value::Number(self.duration / 1000));
        column
    }
}
impl ToGrafana for ::api::report::Report {
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
        column.push(Value::Number(self.summary
            .clone()
            .and_then(|summary| {
                summary
                    .get(&::engine::test_result::TestStatus::Success)
                    .map(|v| *v)
            })
            .unwrap_or(0) as i64));
        column.push(Value::Number(self.summary
            .clone()
            .and_then(|summary| {
                summary
                    .get(&::engine::test_result::TestStatus::Failure)
                    .map(|v| *v)
            })
            .unwrap_or(0) as i64));
        column.push(Value::Number(self.summary
            .clone()
            .and_then(|summary| {
                summary
                    .get(&::engine::test_result::TestStatus::Skipped)
                    .map(|v| *v)
            })
            .unwrap_or(0) as i64));
        column
    }
}

pub enum DataQuery {
    FutureSpans(Box<futures::Future<Item = Vec<::opentracing::Span>, Error = MailboxError>>),
    FutureTestResults(
        Box<futures::Future<Item = Vec<::engine::test_result::TestResult>, Error = MailboxError>>,
    ),
    FutureReports(Box<futures::Future<Item = Vec<::api::report::Report>, Error = MailboxError>>),
}
impl futures::Future for DataQuery {
    type Item = FutureData;
    type Error = MailboxError;

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        match self {
            &mut DataQuery::FutureSpans(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
            &mut DataQuery::FutureTestResults(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
            &mut DataQuery::FutureReports(ref mut v) => v.poll().map(|av| match av {
                futures::Async::Ready(v) => futures::Async::Ready(v.into()),
                futures::Async::NotReady => futures::Async::NotReady,
            }),
        }
    }
}

pub enum FutureData {
    FutureSpans(Vec<::opentracing::Span>),
    FutureTestResults(Vec<::engine::test_result::TestResult>),
    FutureReports(Vec<::api::report::Report>),
}
impl From<Vec<::opentracing::Span>> for FutureData {
    fn from(value: Vec<::opentracing::Span>) -> FutureData {
        FutureData::FutureSpans(value)
    }
}
impl From<Vec<::engine::test_result::TestResult>> for FutureData {
    fn from(value: Vec<::engine::test_result::TestResult>) -> FutureData {
        FutureData::FutureTestResults(value)
    }
}
impl From<Vec<::api::report::Report>> for FutureData {
    fn from(value: Vec<::api::report::Report>) -> FutureData {
        FutureData::FutureReports(value)
    }
}
