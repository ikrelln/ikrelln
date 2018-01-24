table! {
    ingest (id) {
        id -> Nullable<Text>,
        created_at -> Text,
        processed_at -> Nullable<Text>,
    }
}

table! {
    span (id) {
        trace_id -> Text,
        parent_id -> Nullable<Text>,
        id -> Text,
        name -> Nullable<Text>,
        duration -> BigInt,
        ts -> BigInt,
    }
}

table! {
    test (id) {
        id -> Nullable<Text>,
        name -> Text,
    }
}

table! {
    test_result (id) {
        id -> Nullable<Text>,
        test_id -> Text,
        result -> Text,
        duration -> BigInt,
        ts -> BigInt,
    }
}

allow_tables_to_appear_in_same_query!(ingest, span, test, test_result,);
