table! {
    ingest (id) {
        id -> Nullable<Text>,
        created_at -> Text,
        processed_at -> Nullable<Text>,
    }
}

table! {
    ingest_events (id) {
        id -> Nullable<Text>,
        ingest_id -> Text,
        event_type -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    ingest,
    ingest_events,
);
