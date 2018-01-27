use diesel;
use actix::{Handler, MessageResult, ResponseType};
use diesel::prelude::*;
use uuid;
use std::collections::HashMap;

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
        name: span.name,
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
            service_name: endpoint.service_name,
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
            service_name: endpoint.service_name,
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
                name: key.clone(),
                value: value.clone(),
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
                .filter(service_name.eq(le.service_name.clone()))
                .first::<EndpointDb>(&self.0)
                .ok()
            {
                Some(existing) => Some(existing.endpoint_id),
                None => {
                    let new_id = uuid::Uuid::new_v4().hyphenated().to_string();
                    diesel::insert_into(endpoint)
                        .values(&EndpointDb {
                            endpoint_id: new_id.clone(),
                            service_name: le.service_name,
                            ipv4: le.ipv4,
                            ipv6: le.ipv6,
                            port: le.port,
                        })
                        .execute(&self.0)
                        .expect("Error inserting Endpoint");
                    Some(new_id)
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

        use super::schema::span::dsl::*;
        diesel::insert_into(span)
            .values(&to_upsert.span_db)
            .execute(&self.0)
            .expect("Error inserting Span");

        use super::schema::annotation::dsl::*;
        to_upsert.annotations.iter().for_each(|item| {
            diesel::insert_into(annotation)
                .values(item)
                .execute(&self.0)
                .expect("Error inserting annotation");
        });

        use super::schema::tag::dsl::*;
        to_upsert.tags.iter().for_each(|item| {
            diesel::insert_into(tag)
                .values(item)
                .execute(&self.0)
                .expect("Error inserting tag");
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
                .limit(100)
                .load::<EndpointDb>(&self.0)
                .ok()
                .unwrap_or(vec![])
                .iter()
                .map(|ep| ep.service_name.clone().unwrap_or("".to_string()))
                .collect(),
        )
    }
}


pub struct SpanQuery {
    pub service_name: Option<String>,
}

pub struct GetSpans(pub SpanQuery);
impl ResponseType for GetSpans {
    type Item = Vec<::engine::span::Span>;
    type Error = ();
}

impl Handler<GetSpans> for super::DbExecutor {
    type Result = MessageResult<GetSpans>;

    fn handle(&mut self, msg: GetSpans, _: &mut Self::Context) -> Self::Result {
        let target_ep: Option<EndpointDb> = {
            use super::schema::endpoint::dsl::*;

            msg.0.service_name.and_then(|query_service_name| {
                endpoint
                    .filter(service_name.eq(query_service_name))
                    .first::<EndpointDb>(&self.0)
                    .ok()
            })
        };

        let spans: Vec<SpanDb> = {
            use super::schema::span::dsl::*;

            let mut query = span.filter(duration.is_not_null()).into_boxed();

            if let Some(target_ep) = target_ep {
                query = query.filter(
                    remote_endpoint_id
                        .eq(target_ep.endpoint_id.clone())
                        .or(local_endpoint_id.eq(target_ep.endpoint_id)),
                );
            }
            query
                .order(ts.desc())
                .limit(100)
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
                        tags: tags,
                    }
                })
                .collect(),
        )
    }
}
