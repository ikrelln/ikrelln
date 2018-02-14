table! {
    annotation (annotation_id) {
        annotation_id -> Varchar,
        trace_id -> Varchar,
        span_id -> Varchar,
        ts -> Timestamp,
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
        created_at -> Timestamp,
        processed_at -> Nullable<Timestamp>,
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
        ts -> Nullable<Timestamp>,
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
    test_execution (test_id, trace_id) {
        test_id -> Varchar,
        trace_id -> Varchar,
        date -> Timestamp,
        result -> Int4,
        duration -> Int8,
        environment -> Nullable<Varchar>,
    }
}

table! {
    test_item (id) {
        id -> Varchar,
        parent_id -> Nullable<Varchar>,
        name -> Varchar,
        source -> Int4,
    }
}

joinable!(test_execution -> test_item (test_id));

allow_tables_to_appear_in_same_query!(
    annotation,
    endpoint,
    ingest,
    span,
    tag,
    test_execution,
    test_item,
);
