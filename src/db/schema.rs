table! {
    ingest (id) {
        id -> Nullable<Text>,
        created_at -> Text,
        processed_at -> Nullable<Text>,
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

allow_tables_to_appear_in_same_query!(
    ingest,
    test,
    test_result,
);
