use actix_web::*;
use futures::Future;
use futures;

use super::{errors, AppState};

#[derive(Debug, Deserialize)]
struct Search {
    target: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum SearchResponse {
    Node(String),
    //    Leaf { target: String, value: i32 },
}

pub fn setup(_req: HttpRequest<AppState>) -> String {
    String::from(::engine::hello())
}

pub fn search(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.json()
        .from_err()
        .and_then(move |_val: Search| {
            let resp = vec![
                SearchResponse::Node("span".to_string()),
                SearchResponse::Node("test_result".to_string()),
            ];
            Ok(httpcodes::HTTPOk.build().json(resp)?)
        })
        .responder()
}

#[derive(Debug, Deserialize)]
struct TimeRangeRaw {
    from: String,
    to: String,
}
#[derive(Debug, Deserialize)]
struct TimeRange {
    from: String,
    to: String,
    raw: TimeRangeRaw,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Target {
    target: String,
    ref_id: String,
    #[serde(rename = "type")] target_type: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Query {
    timezone: String,
    panel_id: u8,
    interval: String,
    interval_ms: u32,
    max_data_points: u32,
    range: TimeRange,
    range_raw: TimeRangeRaw,
    targets: Vec<Target>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Value {
    Number(i64),
    String(String),
}
#[derive(Debug, Serialize)]
struct Column {
    text: &'static str,
    #[serde(rename = "type")] column_type: &'static str,
}
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum QueryResponse {
    #[serde(rename = "table")]
    Table {
        columns: Vec<Column>,
        rows: Vec<Vec<Value>>,
    },
}

impl ::engine::span::Span {
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
impl ::engine::test::TestResult {
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

use actix;

enum DataQuery {
    FutureSpans(
        Box<futures::Future<Item = Vec<::engine::span::Span>, Error = actix::MailboxError>>,
    ),
    FutureTestResults(
        Box<futures::Future<Item = Vec<::engine::test::TestResult>, Error = actix::MailboxError>>,
    ),
}
impl futures::Future for DataQuery {
    type Item = FutureData;
    type Error = actix::MailboxError;

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
        }
    }
}
enum FutureData {
    FutureSpans(Vec<::engine::span::Span>),
    FutureTestResults(Vec<::engine::test::TestResult>),
}
impl From<Vec<::engine::span::Span>> for FutureData {
    fn from(value: Vec<::engine::span::Span>) -> FutureData {
        FutureData::FutureSpans(value)
    }
}
impl From<Vec<::engine::test::TestResult>> for FutureData {
    fn from(value: Vec<::engine::test::TestResult>) -> FutureData {
        FutureData::FutureTestResults(value)
    }
}

pub fn query(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = errors::IkError>> {
    req.json()
        .from_err()
        .and_then(move |val: Query| {
            let mut data_reqs = vec![];
            for target in &val.targets {
                match target.target.as_ref() {
                    "span" => {
                        let data_req = ::DB_EXECUTOR_POOL.send(::db::span::GetSpans(
                            ::db::span::SpanQuery::default().with_limit(50),
                        ));
                        data_reqs.push(DataQuery::FutureSpans(Box::new(data_req)));
                    }
                    "test_result" => {
                        let data_req = ::DB_EXECUTOR_POOL.send(::db::test::GetTestResults(
                            ::db::test::TestResultQuery::default(),
                        ));
                        data_reqs.push(DataQuery::FutureTestResults(Box::new(data_req)));
                    }
                    _ => (),
                }
            }

            futures::future::join_all(data_reqs)
                .from_err()
                .and_then(|res| {
                    let mut responses = vec![];
                    for data in res {
                        match data {
                            FutureData::FutureSpans(spans) => {
                                let mut columns = vec![];
                                for span in spans {
                                    columns.push(span.as_columns());
                                }
                                let response = QueryResponse::Table {
                                    columns: ::engine::span::Span::as_column_types(),
                                    rows: columns,
                                };
                                responses.push(response);
                            }
                            FutureData::FutureTestResults(test_results) => {
                                let mut columns = vec![];
                                for test_result in test_results {
                                    columns.push(test_result.as_columns());
                                }
                                let response = QueryResponse::Table {
                                    columns: ::engine::test::TestResult::as_column_types(),
                                    rows: columns,
                                };
                                responses.push(response);
                            }
                        }
                    }
                    Ok(httpcodes::HTTPOk.build().json(responses)?)
                })
        })
        .responder()
}
