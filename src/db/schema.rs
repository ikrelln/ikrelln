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
    report (id) {
        id -> Varchar,
        name -> Varchar,
        folder -> Varchar,
        created_on -> Timestamp,
        last_update -> Timestamp,
    }
}

table! {
    script (id) {
        id -> Varchar,
        name -> Varchar,
        source -> Varchar,
        script_type -> Int4,
        date_added -> Timestamp,
        status -> Int4,
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
    test_item (id) {
        id -> Varchar,
        parent_id -> Varchar,
        name -> Varchar,
        source -> Int4,
    }
}

table! {
    test_result (test_id, trace_id) {
        test_id -> Varchar,
        trace_id -> Varchar,
        date -> Timestamp,
        status -> Int4,
        duration -> Int8,
        environment -> Nullable<Varchar>,
        components_called -> Varchar,
        nb_spans -> Int4,
    }
}

table! {
    test_result_in_report (report_id, test_id, trace_id, category) {
        report_id -> Varchar,
        test_id -> Varchar,
        trace_id -> Varchar,
        category -> Varchar,
        environment -> Nullable<Varchar>,
        status -> Int4,
    }
}

joinable!(test_result -> test_item (test_id));
joinable!(test_result_in_report -> report (report_id));

allow_tables_to_appear_in_same_query!(
    annotation,
    endpoint,
    report,
    script,
    span,
    tag,
    test_item,
    test_result,
    test_result_in_report,
);
