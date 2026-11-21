//! GitHub scale sets and unique advisory lock keys.

diesel::table! {
    scale_sets (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        organization_id -> Uuid,
        label_id -> Uuid,
        github_id -> Int8,
        runner_group_id -> Int8,
        lock_key -> Int8,
        remote_removed_at -> Nullable<Timestamptz>,
    }
}
