use actix::prelude::Message;
use actix::{Handler, MessageResult};
use chrono;
use diesel;
use diesel::prelude::*;
use uuid;

use crate::db::schema::span;
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

use crate::db::schema::endpoint;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "endpoint"]
pub struct EndpointDb {
    endpoint_id: String,
    service_name: Option<String>,
    ipv4: Option<String>,
    ipv6: Option<String>,
    port: Option<i32>,
}

use crate::db::schema::tag;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "tag"]
pub struct TagDb {
    span_id: String,
    name: String,
    value: String,
}

use crate::db::schema::annotation;
#[derive(Debug, Insertable, Queryable)]
#[table_name = "annotation"]
pub struct AnnotationDb {
    annotation_id: String,
    trace_id: String,
    span_id: String,
    ts: chrono::NaiveDateTime,
    value: String,
}

struct FromSpan {
    span_db: SpanDb,
    local_endpoint: Option<EndpointDb>,
    remote_endpoint: Option<EndpointDb>,
    tags: Vec<TagDb>,
    annotations: Vec<AnnotationDb>,
}

fn get_all_from_span(span: &crate::opentracing::Span) -> FromSpan {
    let trace_id = span.trace_id.clone();
    let span_id = span.id.clone();

    let span_db = SpanDb {
        trace_id: trace_id.clone(),
        id: span_id.clone(),
        parent_id: span.parent_id.clone(),
        name: span.name.clone().map(|s| s.to_lowercase()),
        kind: span.kind.clone().map(|k| k.to_string()),
        duration: span.duration,
        ts: span.timestamp.map(|ts| {
            // span timestamp is in microseconds
            chrono::NaiveDateTime::from_timestamp(
                ts / 1000 / 1000,
                (ts % (1000 * 1000) * 1000) as u32,
            )
        }),
        debug: span.debug,
        shared: span.shared,
        local_endpoint_id: None,
        remote_endpoint_id: None,
    };

    let local_endpoint = if let Some(endpoint) = span.local_endpoint.clone() {
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

    let remote_endpoint = if let Some(endpoint) = span.remote_endpoint.clone() {
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

    let annotations = span
        .annotations
        .iter()
        .map(|annotation| {
            AnnotationDb {
                trace_id: trace_id.clone(),
                span_id: span_id.clone(),
                annotation_id: uuid::Uuid::new_v4().to_hyphenated().to_string(),
                ts: chrono::NaiveDateTime::from_timestamp(
                    // timestamp is in microseconds
                    annotation.timestamp / 1000 / 1000,
                    (annotation.timestamp % 1000 * 1000) as u32,
                ),
                value: annotation.value.clone(),
            }
        })
        .collect();

    let tags = span
        .tags
        .iter()
        .map(|(key, value)| TagDb {
            span_id: span_id.clone(),
            name: key.clone().to_lowercase(),
            value: value.clone(),
        })
        .collect();

    FromSpan {
        span_db,
        local_endpoint,
        remote_endpoint,
        annotations,
        tags,
    }
}

impl Message for crate::opentracing::Span {
    type Result = crate::opentracing::Span;
}

impl super::DbExecutor {
    fn find_endpoint(&mut self, ep: &EndpointDb) -> Option<EndpointDb> {
        use super::super::schema::endpoint::dsl::*;

        let mut query = endpoint.into_boxed();
        if let Some(query_service_name) = ep.service_name.clone() {
            query = query.filter(service_name.eq(query_service_name));
        }
        if let Some(query_ipv4) = ep.ipv4.clone() {
            query = query.filter(ipv4.eq(query_ipv4));
        }
        if let Some(query_ipv6) = ep.ipv6.clone() {
            query = query.filter(ipv6.eq(query_ipv6));
        }
        if let Some(query_port) = ep.port {
            query = query.filter(port.eq(query_port));
        }

        query
            .first::<EndpointDb>(self.0.as_ref().expect("fail to get DB"))
            .ok()
    }

    fn upsert_endpoint(&mut self, ep: Option<EndpointDb>) -> Option<String> {
        if let Some(le) = ep {
            use super::super::schema::endpoint::dsl::*;

            match self.find_endpoint(&le) {
                Some(existing) => Some(existing.endpoint_id),
                None => {
                    let new_id = uuid::Uuid::new_v4().to_hyphenated().to_string();
                    let could_insert = diesel::insert_into(endpoint)
                        .values(&EndpointDb {
                            endpoint_id: new_id.clone(),
                            service_name: le.service_name.clone(),
                            ipv4: le.ipv4.clone(),
                            ipv6: le.ipv6.clone(),
                            port: le.port,
                        })
                        .execute(self.0.as_ref().expect("fail to get DB"));
                    if could_insert.is_err() {
                        self.find_endpoint(&le).map(|existing| existing.endpoint_id)
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

impl Handler<crate::opentracing::Span> for super::DbExecutor {
    type Result = MessageResult<crate::opentracing::Span>;

    fn handle(&mut self, msg: crate::opentracing::Span, ctx: &mut Self::Context) -> Self::Result {
        self.check_db_connection(ctx);

        let mut to_upsert = get_all_from_span(&msg);

        to_upsert.span_db.local_endpoint_id = self.upsert_endpoint(to_upsert.local_endpoint);
        to_upsert.span_db.remote_endpoint_id = self.upsert_endpoint(to_upsert.remote_endpoint);

        let _span_in_db = {
            use super::super::schema::span::dsl::*;
            match span
                .filter(
                    id.eq(&to_upsert.span_db.id)
                        .and(trace_id.eq(&to_upsert.span_db.trace_id)),
                )
                .first::<SpanDb>(self.0.as_ref().expect("fail to get DB"))
            {
                Ok(_) => {
                    //TODO: manage more update cases than duration
                    diesel::update(
                        span.filter(
                            id.eq(&to_upsert.span_db.id)
                                .and(trace_id.eq(&to_upsert.span_db.trace_id)),
                        ),
                    )
                    .set(duration.eq(to_upsert.span_db.duration))
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .map_err(|err| self.reconnect_if_needed(ctx, &err))
                }
                Err(_) => diesel::insert_into(span)
                    .values(&to_upsert.span_db)
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .map_err(|err| self.reconnect_if_needed(ctx, &err)),
            }
        };

        {
            use super::super::schema::annotation::dsl::*;
            to_upsert.annotations.iter().for_each(|item| {
                diesel::insert_into(annotation)
                    .values(item)
                    .execute(self.0.as_ref().expect("fail to get DB"))
                    .ok();
            });
        }

        {
            use super::super::schema::tag::dsl::*;
            let existing_tags = tag
                .select(name)
                .filter(
                    span_id
                        .eq(to_upsert.span_db.id)
                        .and(name.eq_any(to_upsert.tags.iter().map(|item| item.name.clone()))),
                )
                .load::<String>(self.0.as_ref().expect("fail to get DB"))
                .ok()
                .unwrap_or_else(|| vec![]);
            to_upsert.tags.iter().for_each(|item| {
                if existing_tags.contains(&item.name) {
                    diesel::update(tag.filter(span_id.eq(&item.span_id).and(name.eq(&item.name))))
                        .set(value.eq(&item.value))
                        .execute(self.0.as_ref().expect("fail to get DB"))
                        .ok();
                } else {
                    diesel::insert_into(tag)
                        .values(item)
                        .execute(self.0.as_ref().expect("fail to get DB"))
                        .ok();
                }
            });
        }
        MessageResult(msg)
    }
}
