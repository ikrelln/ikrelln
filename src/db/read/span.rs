use actix::prelude::Message;
use actix::{Handler, MessageResult};
use actix_web;
use chrono;
use diesel::prelude::*;
use std::collections::HashMap;
use std::str::FromStr;

static SPAN_QUERY_LIMIT: i64 = 500;
use db::schema::span;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "span"]
pub struct SpanDb {
    pub trace_id: String,
    pub id: String,
    parent_id: Option<String>,
    name: Option<String>,
    kind: Option<String>,
    duration: Option<i64>,
    ts: Option<chrono::NaiveDateTime>,
    debug: bool,
    shared: bool,
    local_endpoint_id: Option<String>,
    remote_endpoint_id: Option<String>,
}

static ENDPOINT_QUERY_LIMIT: i64 = 1000;
use db::schema::endpoint;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "endpoint"]
pub struct EndpointDb {
    endpoint_id: String,
    service_name: Option<String>,
    ipv4: Option<String>,
    ipv6: Option<String>,
    port: Option<i32>,
}

static TAG_QUERY_LIMIT: i64 = 100;
use db::schema::tag;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "tag"]
pub struct TagDb {
    span_id: String,
    name: String,
    value: String,
}

static ANNOTATION_QUERY_LIMIT: i64 = 100;
use db::schema::annotation;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "annotation"]
pub struct AnnotationDb {
    annotation_id: String,
    trace_id: String,
    span_id: String,
    ts: chrono::NaiveDateTime,
    value: String,
}

pub struct GetServices;
impl Message for GetServices {
    type Result = Vec<String>;
}

impl Handler<GetServices> for super::DbReadExecutor {
    type Result = MessageResult<GetServices>;

    fn handle(&mut self, _msg: GetServices, _: &mut Self::Context) -> Self::Result {
        use super::super::schema::endpoint::dsl::*;

        MessageResult(
            endpoint
                .limit(ENDPOINT_QUERY_LIMIT)
                .order(service_name.asc())
                .load::<EndpointDb>(self.0.as_ref().expect("fail to get DB"))
                .ok()
                .unwrap_or_else(|| vec![])
                .iter()
                .map(|ep| ep.service_name.clone().unwrap_or_else(|| "".to_string()))
                .collect(),
        )
    }
}

#[derive(Debug)]
pub struct SpanQuery {
    pub filter_finish: bool,
    pub service_name: Option<String>,
    pub span_name: Option<String>,
    pub trace_id: Option<String>,
    pub min_duration: Option<i64>,
    pub max_duration: Option<i64>,
    pub end_ts: chrono::NaiveDateTime,
    pub lookback: Option<chrono::Duration>,
    pub limit: i64,
    pub only_endpoint: bool,
}

impl Default for SpanQuery {
    fn default() -> Self {
        SpanQuery {
            filter_finish: true,
            service_name: None,
            span_name: None,
            trace_id: None,
            min_duration: None,
            max_duration: None,
            end_ts: chrono::Utc::now().naive_utc(),
            lookback: None,
            limit: SPAN_QUERY_LIMIT,
            only_endpoint: false,
        }
    }
}

impl SpanQuery {
    pub fn from_req(req: &actix_web::HttpRequest<::api::AppState>) -> Self {
        SpanQuery {
            filter_finish: req.query()
                .get("finished")
                .and_then(|s| FromStr::from_str(s).ok())
                .unwrap_or(true),
            service_name: req.query().get("serviceName").map(|s| s.to_string()),
            span_name: req.query().get("spanName").map(|s| s.to_string()),
            trace_id: req.query().get("traceId").map(|s| s.to_string()),
            min_duration: req.query()
                .get("minDuration")
                .and_then(|s| s.parse::<i64>().ok()),
            max_duration: req.query()
                .get("maxDuration")
                .and_then(|s| s.parse::<i64>().ok()),
            end_ts: req.query()
                .get("endTs")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|v| {
                    // query timestamp is in milliseconds
                    chrono::NaiveDateTime::from_timestamp(
                        v / 1000,
                        ((v % 1000) * 1000 * 1000) as u32,
                    )
                })
                .unwrap_or_else(|| chrono::Utc::now().naive_utc()),
            lookback: req.query()
                .get("lookback")
                .and_then(|s| s.parse::<i64>().ok())
                .map(chrono::Duration::milliseconds),
            limit: req.query()
                .get("limit")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|v| {
                    if v > SPAN_QUERY_LIMIT {
                        SPAN_QUERY_LIMIT
                    } else {
                        v
                    }
                })
                .unwrap_or(SPAN_QUERY_LIMIT),
            only_endpoint: false,
        }
    }

    pub fn with_trace_id(self, trace_id: String) -> Self {
        SpanQuery {
            trace_id: Some(trace_id),
            ..self
        }
    }
    pub fn with_limit(self, limit: i64) -> Self {
        SpanQuery { limit, ..self }
    }
    pub fn only_endpoint(self) -> Self {
        SpanQuery {
            only_endpoint: true,
            ..self
        }
    }
}

pub struct GetSpans(pub SpanQuery);
impl Message for GetSpans {
    type Result = Vec<::opentracing::Span>;
}

impl Handler<GetSpans> for super::DbReadExecutor {
    type Result = MessageResult<GetSpans>;

    fn handle(&mut self, msg: GetSpans, _: &mut Self::Context) -> Self::Result {
        let query_endpoint: Option<Result<EndpointDb, _>> = {
            use super::super::schema::endpoint::dsl::*;

            msg.0.service_name.map(|query_service_name| {
                endpoint
                    .filter(service_name.eq(query_service_name.to_lowercase()))
                    .first::<EndpointDb>(self.0.as_ref().expect("fail to get DB"))
            })
        };
        if let Some(Err(_err)) = query_endpoint {
            // no endpoint found matching query
            return MessageResult(vec![]);
        }

        let spans: Vec<SpanDb> = {
            use super::super::schema::span::dsl::*;

            let mut query = span.into_boxed();

            if msg.0.filter_finish {
                query = query.filter(duration.is_not_null());
            }

            if let Some(Ok(query_endpoint)) = query_endpoint {
                query = query.filter(
                    remote_endpoint_id
                        .eq(query_endpoint.endpoint_id.clone())
                        .or(local_endpoint_id.eq(query_endpoint.endpoint_id)),
                );
            }

            if let Some(query_span_name) = msg.0.span_name {
                query = query.filter(name.eq(query_span_name));
            }

            if let Some(query_trace_id) = msg.0.trace_id {
                query = query.filter(trace_id.eq(query_trace_id));
            }

            if let Some(query_max_duration) = msg.0.max_duration {
                query = query.filter(duration.le(query_max_duration));
            }
            if let Some(query_min_duration) = msg.0.min_duration {
                query = query.filter(duration.ge(query_min_duration));
            }

            query = query.filter(ts.le(msg.0.end_ts));
            if let Some(query_lookback) = msg.0.lookback {
                query = query.filter(ts.ge(msg.0.end_ts - query_lookback));
            }

            if msg.0.only_endpoint {
                query = query.filter(remote_endpoint_id.is_not_null());
            }

            query
                .order(ts.asc())
                .limit(msg.0.limit)
                .load::<SpanDb>(self.0.as_ref().expect("fail to get DB"))
                .ok()
                .unwrap_or_else(|| vec![])
        };

        let without_tags = msg.0.only_endpoint;
        let without_annotations = msg.0.only_endpoint;

        let mut endpoint_cache = super::super::helper::Cacher::new();

        MessageResult(
            spans
                .iter()
                .map(|spandb| {
                    let local_endpoint = spandb.local_endpoint_id.clone().and_then(|lep_id| {
                        endpoint_cache
                            .get(&lep_id, |id| {
                                use super::super::schema::endpoint::dsl::*;

                                endpoint
                                    .filter(endpoint_id.eq(id))
                                    .first::<EndpointDb>(self.0.as_ref().expect("fail to get DB"))
                                    .ok()
                                    .map(|ep| ::opentracing::span::Endpoint {
                                        service_name: ep.service_name,
                                        ipv4: ep.ipv4,
                                        ipv6: ep.ipv6,
                                        port: ep.port,
                                    })
                            })
                            .clone()
                    });
                    let remote_endpoint = spandb.remote_endpoint_id.clone().and_then(|rep_id| {
                        endpoint_cache
                            .get(&rep_id, |id| {
                                use super::super::schema::endpoint::dsl::*;

                                endpoint
                                    .filter(endpoint_id.eq(id))
                                    .first::<EndpointDb>(self.0.as_ref().expect("fail to get DB"))
                                    .ok()
                                    .map(|ep| ::opentracing::span::Endpoint {
                                        service_name: ep.service_name,
                                        ipv4: ep.ipv4,
                                        ipv6: ep.ipv6,
                                        port: ep.port,
                                    })
                            })
                            .clone()
                    });

                    let annotations = if !without_annotations {
                        use super::super::schema::annotation::dsl::*;

                        annotation
                            .filter(trace_id.eq(&spandb.trace_id).and(span_id.eq(&spandb.id)))
                            .limit(ANNOTATION_QUERY_LIMIT)
                            .load::<AnnotationDb>(self.0.as_ref().expect("fail to get DB"))
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|an| ::opentracing::span::Annotation {
                                timestamp: ((an.ts.timestamp() * 1000)
                                    + i64::from(an.ts.timestamp_subsec_millis()))
                                    * 1000,
                                value: an.value.clone(),
                            })
                            .collect()
                    } else {
                        vec![]
                    };

                    //TODO: way too slow
                    let tags: HashMap<String, String> = if !without_tags {
                        use super::super::schema::tag::dsl::*;

                        tag.filter(span_id.eq(&spandb.id))
                            .limit(TAG_QUERY_LIMIT)
                            .load::<TagDb>(self.0.as_ref().expect("fail to get DB"))
                            .ok()
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|t| (t.name.clone(), t.value.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    };

                    let binary_annotation_endpoint =
                        remote_endpoint.clone().or_else(|| local_endpoint.clone());

                    ::opentracing::Span {
                        trace_id: spandb.trace_id.clone(),
                        id: spandb.id.clone(),
                        parent_id: spandb.parent_id.clone(),
                        name: spandb.name.clone().map(|s| s.chars().take(250).collect()),
                        kind: spandb.kind.clone().map(|k| k.into()),
                        timestamp: spandb.ts.map(|ts| {
                            ((ts.timestamp() * 1000) + i64::from(ts.timestamp_subsec_millis()))
                                * 1000
                        }),
                        duration: spandb.duration,
                        debug: spandb.debug,
                        shared: spandb.shared,
                        local_endpoint,
                        remote_endpoint,
                        annotations,
                        tags: tags.clone(),
                        binary_annotations: tags.iter()
                            .map(|(k, v)| ::opentracing::span::BinaryTag {
                                key: k.clone(),
                                value: v.clone(),
                                endpoint: binary_annotation_endpoint.clone(),
                            })
                            .collect(),
                    }
                })
                .collect(),
        )
    }
}
