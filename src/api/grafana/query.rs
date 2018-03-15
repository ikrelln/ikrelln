use actix_web::*;
use futures::Future;
use futures;

use super::data_queries::{DataQuery, FutureData};

use super::{errors, AppState};
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
pub enum Value {
    Number(i64),
    String(String),
}
#[derive(Debug, Serialize)]
pub struct Column {
    pub text: &'static str,
    #[serde(rename = "type")] pub column_type: &'static str,
}
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum QueryResponse {
    #[serde(rename = "table")]
    Table {
        columns: Vec<Column>,
        rows: Vec<Vec<Value>>,
    },
}

pub trait ToGrafana {
    fn as_column_types() -> Vec<Column>;
    fn as_columns(self) -> Vec<Value>;
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
                    "spans" => {
                        let data_req = ::DB_EXECUTOR_POOL.send(::db::span::GetSpans(
                            ::db::span::SpanQuery::default().with_limit(50),
                        ));
                        data_reqs.push(DataQuery::FutureSpans(Box::new(data_req)));
                    }
                    "test_results" => {
                        let data_req = ::DB_EXECUTOR_POOL.send(::db::test::GetTestResults(
                            ::db::test::TestResultQuery::default(),
                        ));
                        data_reqs.push(DataQuery::FutureTestResults(Box::new(data_req)));
                    }
                    "reports" => {
                        let data_req = ::DB_EXECUTOR_POOL.send(::db::reports::GetAll);
                        data_reqs.push(DataQuery::FutureReports(Box::new(data_req)));
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
                                    columns: ::opentracing::Span::as_column_types(),
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
                                    columns: ::engine::test_result::TestResult::as_column_types(),
                                    rows: columns,
                                };
                                responses.push(response);
                            }
                            FutureData::FutureReports(reports) => {
                                let mut columns = vec![];
                                for report in reports {
                                    columns.push(report.as_columns());
                                }
                                let response = QueryResponse::Table {
                                    columns: ::api::report::Report::as_column_types(),
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
