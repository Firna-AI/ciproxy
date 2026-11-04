//! Job metadata and retention timestamps, without job contents.

diesel::table! {
    jobs (id) {
        id -> Uuid,
        organization_id -> Uuid,
        github_id -> Int8,
        label_id -> Uuid,
        scale_set_id -> Uuid,
        runner_id -> Nullable<Uuid>,
        name -> Text,
        workflow -> Text,
        repository -> Text,
        git_ref -> Nullable<Text>,
        state -> Text,
        result -> Nullable<Text>,
        queued_at -> Timestamptz,
        started_at -> Nullable<Timestamptz>,
        completed_at -> Nullable<Timestamptz>,
        routing_reason -> Nullable<Text>,
        cache_source -> Nullable<Text>,
        cache_restores -> Nullable<Int8>,
        cache_restored_bytes -> Nullable<Int8>,
    }
}
