//! Routing settings isolated by workspace.

diesel::table! {
    settings (workspace_id) {
        workspace_id -> Uuid,
        overflow_enabled -> Bool,
        overflow_wait -> Text,
        boat_spend_limit_usd_micros -> Nullable<Int8>,
    }
}
