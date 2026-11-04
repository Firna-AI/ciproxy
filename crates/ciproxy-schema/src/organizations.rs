//! Organization ownership and GitHub installation state.

diesel::table! {
    organizations (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        github_id -> Int8,
        login -> Text,
        installation_id -> Int8,
        app_state -> Text,
        repository_mode -> Text,
        allowed_repository_ids -> Array<Int8>,
        removed_at -> Nullable<Timestamptz>,
        delete_after -> Nullable<Timestamptz>,
    }
}
