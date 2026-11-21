//! The customer tenant that owns shared execution resources.

diesel::table! {
    workspaces (id) {
        id -> Uuid,
        name -> Text,
        delete_after -> Nullable<Timestamptz>,
        metering_started_at -> Nullable<Timestamptz>,
        metering_retained_from -> Nullable<Date>,
    }
}
