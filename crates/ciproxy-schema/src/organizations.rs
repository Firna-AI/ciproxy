//! Organization ownership and GitHub installation state.

diesel::table! {
    organizations (id) {
        id -> Uuid,
        github_id -> Int8,
        login -> Text,
        installation_id -> Int8,
        app_state -> Text,
        removed_at -> Nullable<Timestamptz>,
        delete_after -> Nullable<Timestamptz>,
    }
}
