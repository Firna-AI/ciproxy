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
    billing_payments (command_id) {
        command_id -> Uuid,
        workspace_id -> Uuid,
        state_json -> Text,
        pending -> Bool,
    }
}

diesel::table! {
    billing_quotes (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        quote_json -> Text,
        expires_at -> Timestamptz,
        command_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    billing_events (event_id) {
        event_id -> Text,
        workspace_id -> Uuid,
        applied_at -> Timestamptz,
    }
}

diesel::table! {
    billing_renewals (workspace_id, invoice_id) {
        workspace_id -> Uuid,
        subscription_id -> Text,
        invoice_id -> Text,
        outcome_json -> Text,
    }
}
