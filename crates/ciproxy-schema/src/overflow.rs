//! Overflow account observations, durable deliveries, and isolated session locks.

diesel::table! {
    overflow_connection_grants (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        key_id -> Uuid,
    }
}

diesel::table! {
    overflow_accounts (key_id) {
        key_id -> Uuid,
        state_json -> Text,
    }
}

diesel::table! {
    overflow_starts (delivery_id) {
        delivery_id -> Uuid,
        workspace_id -> Uuid,
        runner_id -> Uuid,
        key_id -> Nullable<Uuid>,
        record_json -> Text,
        launch_ciphertext -> Nullable<Bytea>,
        token_hash -> Bytea,
        reported_at -> Nullable<Timestamptz>,
        ends_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    overflow_locks (workspace_id) {
        workspace_id -> Uuid,
        lock_key -> BigInt,
    }
}
