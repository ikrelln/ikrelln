use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use std::collections::HashMap;
use actix_web;
use std::str::FromStr;
use time;

use db::schema::span;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "span"]
pub struct SpanDb {
    trace_id: String,
    id: String,
    parent_id: Option<String>,
    name: Option<String>,
    kind: Option<String>,
    duration: Option<i64>,
    ts: Option<i64>,
    debug: bool,
    shared: bool,
    local_endpoint_id: Option<String>,
    remote_endpoint_id: Option<String>,
}

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

use db::schema::tag;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "tag"]
pub struct TagDb {
    tag_id: String,
    trace_id: String,
    span_id: String,
    name: String,
    value: String,
}

use db::schema::annotation;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "annotation"]
pub struct AnnotationDb {
    annotation_id: String,
    trace_id: String,
    span_id: String,
    ts: i64,
    value: String,
}

struct FromSpan {
    span_db: SpanDb,
    local_endpoint: Option<EndpointDb>,
    remote_endpoint: Option<EndpointDb>,
    tags: Vec<TagDb>,
    annotations: Vec<AnnotationDb>,
}

fn get_all_from_span(span: ::engine::span::Span) -> FromSpan {
    let trace_id = span.trace_id;
    let span_id = span.id;

    let span_db = SpanDb {
        trace_id: trace_id.clone(),
        id: span_id.clone(),
        parent_id: span.parent_id,
        name: span.name.map(|s| s.to_lowercase()),
        kind: span.kind.map(|k| k.to_string()),
        duration: span.duration,
        ts: span.timestamp,
        debug: span.debug,
        shared: span.shared,
        local_endpoint_id: None,
        remote_endpoint_id: None,
    };

    let local_endpoint = if let Some(endpoint) = span.local_endpoint {
        Some(EndpointDb {
            endpoint_id: "n/a".to_string(),
            service_name: endpoint.service_name.map(|s| s.to_lowercase()),
            ipv4: endpoint.ipv4,
            ipv6: endpoint.ipv6,
            port: endpoint.port,
        })
    } else {
        None
    };

    let remote_endpoint = if let Some(endpoint) = span.remote_endpoint {
        Some(EndpointDb {
            endpoint_id: "n/a".to_string(),
            service_name: endpoint.service_name.map(|s| s.to_lowercase()),
            ipv4: endpoint.ipv4,
            ipv6: endpoint.ipv6,
            port: endpoint.port,
        })
    } else {
        None
    };

    let annotations = span.annotations
        .iter()
        .map(|annotation| {
            AnnotationDb {
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                annotation_id: uuid::Uuid::new_v4().hyphenated().to_string(),
                ts: annotation.timestamp,
                value: annotation.value.clone(),
            }
        })
        .collect();

    let tags = span.tags
        .iter()
        .map(|(key, value)| {
            TagDb {
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                tag_id: uuid::Uuid::new_v4().hyphenated().to_string(),
                name: key.clone().to_lowercase(),
                value: value.clone().to_lowercase(),
            }
        })
        .collect();

    FromSpan {
        span_db: span_db,
        local_endpoint: local_endpoint,
        remote_endpoint: remote_endpoint,
        annotations: annotations,
        tags: tags,
    }
}

impl ResponseType for ::engine::span::Span {
    type Item = ();
    type Error = ();
}

impl super::DbExecutor {
    fn upsert_endpoint(&mut self, ep: Option<EndpointDb>) -> Option<String> {
        if let Some(le) = ep {
            use super::schema::endpoint::dsl::*;

            match endpoint
                .filter(
                    service_name
                        .eq(le.service_name.clone())
                        .and(ipv4.eq(le.ipv4.clone())),
                )
                .first::<EndpointDb>(&self.0)
                .ok()
            {
                Some(existing) => Some(existing.endpoint_id),
                None => {
                    let new_id = uuid::Uuid::new_v4().hyphenated().to_string();
                    let could_insert = diesel::insert_into(endpoint)
                        .values(&EndpointDb {
                            endpoint_id: new_id.clone(),
                            service_name: le.service_name.clone(),
                            ipv4: le.ipv4.clone(),
                            ipv6: le.ipv6,
                            port: le.port,
                        })
                        .execute(&self.0);
                    if let Err(_) = could_insert {
                        endpoint
                            .filter(
                                service_name
                                    .eq(le.service_name.clone())
                                    .and(ipv4.eq(le.ipv4.clone())),
                            )
                            .first::<EndpointDb>(&self.0)
                            .ok()
                            .map(|existing| existing.endpoint_id)
                    } else {
                        Some(new_id)
                    }
                }
            }
        } else {
            None
        }
    }
}

impl Handler<::engine::span::Span> for super::DbExecutor {
    type Result = MessageResult<::engine::span::Span>;

    fn handle(&mut self, msg: ::engine::span::Span, _: &mut Self::Context) -> Self::Result {
        let mut to_upsert = get_all_from_span(msg);

        to_upsert.span_db.local_endpoint_id = self.upsert_endpoint(to_upsert.local_endpoint);
        to_upsert.span_db.remote_endpoint_id = self.upsert_endpoint(to_upsert.remote_endpoint);

        {
            use super::schema::span::dsl::*;
            match span.filter(
                id.eq(to_upsert.span_db.id.clone())
                    .and(trace_id.eq(to_upsert.span_db.trace_id.clone())),
            ).first::<SpanDb>(&self.0)
            {
                Ok(_) => {
                    //TODO: manage more update cases than duration
                    diesel::update(
                        span.filter(
                            id.eq(to_upsert.span_db.id.clone())
                                .and(trace_id.eq(to_upsert.span_db.trace_id.clone())),
                        ),
                    ).set(duration.eq(to_upsert.span_db.duration))
                        .execute(&self.0)
                        .expect(&format!("Error updating Span for {:?}", to_upsert.span_db));
                }
                Err(_) => {
                    diesel::insert_into(span)
                        .values(&to_upsert.span_db)
                        .execute(&self.0)
                        .expect(&format!("Error inserting Span for {:?}", to_upsert.span_db));
                }
            };
        }

        use super::schema::annotation::dsl::*;
        to_upsert.annotations.iter().for_each(|item| {
            diesel::insert_into(annotation)
                .values(item)
                .execute(&self.0)
                .expect(&format!("Error inserting annotation {:?}", item));
        });

        use super::schema::tag::dsl::*;
        to_upsert.tags.iter().for_each(|item| {
            diesel::insert_into(tag)
                .values(item)
                .execute(&self.0)
                .expect(&format!("Error inserting tag {:?}", item));
        });

        Ok(())
    }
}


pub struct GetServices;
impl ResponseType for GetServices {
    type Item = Vec<String>;
    type Error = ();
}

impl Handler<GetServices> for super::DbExecutor {
    type Result = MessageResult<GetServices>;

    fn handle(&mut self, _msg: GetServices, _: &mut Self::Context) -> Self::Result {
        use super::schema::endpoint::dsl::*;

        Ok(
            endpoint
                .order(service_name.asc())
                .load::<EndpointDb>(&self.0)
                .ok()
                .unwrap_or(vec![])
                .iter()
                .map(|ep| ep.service_name.clone().unwrap_or("".to_string()))
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
    pub end_ts: i64,
    pub lookback: Option<i64>,
    pub limit: i64,
}

impl SpanQuery {
    pub fn from_req(req: &actix_web::HttpRequest<::http::AppState>) -> Self {
        return SpanQuery {
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
                .map(|v| v * 1000)
                .unwrap_or_else(|| {
                    let now = time::get_time();
                    (now.sec * 1000000) + ((now.nsec / 1000) as i64)
                }),
            lookback: req.query()
                .get("lookback")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|v| v * 1000),
            limit: req.query()
                .get("limit")
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(100),
        };
    }

    pub fn with_trace_id(self, trace_id: String) -> Self {
        SpanQuery {
            trace_id: Some(trace_id),
            ..self
        }
    }
}

pub struct GetSpans(pub SpanQuery);
impl ResponseType for GetSpans {
    type Item = Vec<::engine::span::Span>;
    type Error = ();
}

impl Handler<GetSpans> for super::DbExecutor {
    type Result = MessageResult<GetSpans>;

    fn handle(&mut self, msg: GetSpans, _: &mut Self::Context) -> Self::Result {
        let query_endpoint: Option<Result<EndpointDb, _>> = {
            use super::schema::endpoint::dsl::*;

            msg.0.service_name.map(|query_service_name| {
                endpoint
                    .filter(service_name.eq(query_service_name.to_lowercase()))
                    .first::<EndpointDb>(&self.0)
            })
        };
        if let Some(Err(_err)) = query_endpoint {
            // no endpoint found matching query
            return Ok(vec![]);
        }

        let spans: Vec<SpanDb> = {
            use super::schema::span::dsl::*;

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

            query
                .order(ts.asc())
                .limit(msg.0.limit)
                .load::<SpanDb>(&self.0)
                .ok()
                .unwrap_or(vec![])
        };

        Ok(
            spans
                .iter()
                .map(|spandb| {
                    let local_endpoint = spandb.local_endpoint_id.clone().and_then(|lep_id| {
                        use super::schema::endpoint::dsl::*;

                        endpoint
                            .filter(endpoint_id.eq(lep_id))
                            .first::<EndpointDb>(&self.0)
                            .ok()
                            .map(|ep| {
                                ::engine::span::Endpoint {
                                    service_name: ep.service_name,
                                    ipv4: ep.ipv4,
                                    ipv6: ep.ipv6,
                                    port: ep.port,
                                }
                            })
                    });
                    let remote_endpoint = spandb.remote_endpoint_id.clone().and_then(|lep_id| {
                        use super::schema::endpoint::dsl::*;

                        endpoint
                            .filter(endpoint_id.eq(lep_id))
                            .first::<EndpointDb>(&self.0)
                            .ok()
                            .map(|ep| {
                                ::engine::span::Endpoint {
                                    service_name: ep.service_name,
                                    ipv4: ep.ipv4,
                                    ipv6: ep.ipv6,
                                    port: ep.port,
                                }
                            })
                    });

                    let annotations = {
                        use super::schema::annotation::dsl::*;

                        annotation
                            .filter(
                                trace_id
                                    .eq(spandb.trace_id.clone())
                                    .and(span_id.eq(spandb.id.clone())),
                            )
                            .load::<AnnotationDb>(&self.0)
                            .ok()
                            .unwrap_or(vec![])
                            .iter()
                            .map(|an| {
                                ::engine::span::Annotation {
                                    timestamp: an.ts,
                                    value: an.value.clone(),
                                }
                            })
                            .collect()
                    };

                    //TODO: way too slow
                    let tags: HashMap<String, String> = {
                        use super::schema::tag::dsl::*;

                        tag.filter(
                            trace_id
                                .eq(spandb.trace_id.clone())
                                .and(span_id.eq(spandb.id.clone())),
                        ).load::<TagDb>(&self.0)
                            .ok()
                            .unwrap_or(vec![])
                            .iter()
                            .map(|t| (t.name.clone(), t.value.clone()))
                            .collect()
                    };

                    ::engine::span::Span {
                        trace_id: spandb.trace_id.clone(),
                        id: spandb.id.clone(),
                        parent_id: spandb.parent_id.clone(),
                        name: spandb.name.clone(),
                        kind: spandb.kind.clone().map(|k| k.into()),
                        timestamp: spandb.ts,
                        duration: spandb.duration,
                        debug: spandb.debug,
                        shared: spandb.shared,
                        local_endpoint: local_endpoint,
                        remote_endpoint: remote_endpoint,
                        annotations: annotations,
                        tags: tags.clone(),
                        binary_annotations: tags.iter()
                            .map(|(k, v)| {
                                ::engine::span::Tag {
                                    key: k.clone(),
                                    value: v.clone(),
                                }
                            })
                            .collect(),
                    }
                })
                .collect(),
        )
    }
}
