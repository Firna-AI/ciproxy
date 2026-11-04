//! Confirmed billing state, durable commands, and committed event receipts.

diesel::table! {
    billing_accounts (workspace_id) {
        workspace_id -> Uuid,
        lock_key -> Int8,
        created_at -> Timestamptz,
        state_json -> Nullable<Text>,
        command_json -> Nullable<Text>,
        checkout_json -> Nullable<Text>,
        subscription_id -> Nullable<Text>,
        removal_cancel_requested -> Bool,
    }
}

diesel::table! {
    billing_events (event_id) {
        event_id -> Text,
        workspace_id -> Uuid,
        applied_at -> Timestamptz,
    }
}
