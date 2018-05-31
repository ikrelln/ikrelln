use std::collections::HashSet;

use actix::prelude::*;
use futures::{future, Future};

#[derive(Default)]
pub struct Reporter;
impl Actor for Reporter {
    type Context = Context<Self>;
}
impl actix::Supervised for Reporter {}

impl actix::SystemService for Reporter {
    fn service_started(&mut self, _ctx: &mut Context<Self>) {}
}

#[derive(Message)]
pub struct ResultForReport {
    pub report_group: String,
    pub report_name: String,
    pub category: Option<String>,
    pub result: ::engine::test_result::TestResult,
}

impl Handler<ResultForReport> for Reporter {
    type Result = ();

    fn handle(&mut self, msg: ResultForReport, _ctx: &mut Context<Self>) -> Self::Result {
        ::DB_EXECUTOR_POOL.do_send(msg);
    }
}

#[derive(Hash, PartialEq, Eq)]
pub struct Report {
    pub group: String,
    pub name: String,
    pub category: Option<String>,
}

#[derive(Message)]
pub struct ComputeReportsForResult(pub ::engine::test_result::TestResult);

impl Handler<ComputeReportsForResult> for Reporter {
    type Result = ();

    fn handle(&mut self, msg: ComputeReportsForResult, _ctx: &mut Context<Self>) -> Self::Result {
        Arbiter::handle().spawn(
            ::DB_EXECUTOR_POOL
                .send(::db::span::GetSpans(
                    ::db::span::SpanQuery::default()
                        .with_trace_id(msg.0.trace_id.clone())
                        .with_limit(1000),
                ))
                .then(move |spans| {
                    if let Ok(spans) = spans {
                        let reports_to_send: HashSet<Report> = spans
                            .iter()
                            .filter(|span| span.remote_endpoint.is_some())
                            .map(|span| Report {
                                name: span.remote_endpoint
                                    .clone()
                                    .and_then(|ep| ep.service_name)
                                    .unwrap_or_else(|| "service".to_string()),
                                group: "endpoints".to_string(),
                                category: span.name.clone(),
                            })
                            .collect();
                        reports_to_send.iter().for_each(|report| {
                            actix::Arbiter::system_registry().get::<Reporter>().do_send(
                                ResultForReport {
                                    report_group: report.group.clone(),
                                    report_name: report.name.clone(),
                                    category: report.category.clone(),
                                    result: msg.0.clone(),
                                },
                            )
                        });
                    }
                    future::result(Ok(()))
                }),
        )
    }
}
