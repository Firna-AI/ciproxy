//! The customer tenant that owns shared execution resources.

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        name -> Text,
        delete_after -> Nullable<Timestamptz>,
    }
}
