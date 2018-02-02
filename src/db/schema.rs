table! {
    annotation (annotation_id) {
        annotation_id -> Varchar,
        trace_id -> Varchar,
        span_id -> Varchar,
        ts -> Int8,
        value -> Varchar,
    }
}

table! {
    endpoint (endpoint_id) {
        endpoint_id -> Varchar,
        service_name -> Nullable<Varchar>,
        ipv4 -> Nullable<Varchar>,
        ipv6 -> Nullable<Varchar>,
        port -> Nullable<Int4>,
    }
}

table! {
    ingest (id) {
        id -> Varchar,
        created_at -> Text,
        processed_at -> Nullable<Text>,
    }
}

table! {
    span (trace_id, id) {
        trace_id -> Varchar,
        id -> Varchar,
        parent_id -> Nullable<Varchar>,
        name -> Nullable<Varchar>,
        kind -> Nullable<Varchar>,
        duration -> Nullable<Int8>,
        ts -> Nullable<Int8>,
        debug -> Bool,
        shared -> Bool,
        local_endpoint_id -> Nullable<Varchar>,
        remote_endpoint_id -> Nullable<Varchar>,
    }
}

table! {
    tag (tag_id) {
        tag_id -> Varchar,
        trace_id -> Varchar,
        span_id -> Varchar,
        name -> Varchar,
        value -> Varchar,
    }
}

table! {
    test (id) {
        id -> Varchar,
        name -> Varchar,
    }
}

table! {
    test_result (id) {
        id -> Varchar,
        test_id -> Varchar,
        result -> Varchar,
        duration -> Int8,
        ts -> Int8,
    }
}

allow_tables_to_appear_in_same_query!(
    annotation,
    endpoint,
    ingest,
    span,
    tag,
    test,
    test_result,
);
