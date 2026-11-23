//! Per-provider spend limits and first-offer order.

diesel::table! {
    overflow_provider_settings (workspace_id, provider) {
        workspace_id -> Uuid,
        provider -> Text,
        spend_limit_usd_micros -> Nullable<Int8>,
        offered_position -> Nullable<Int4>,
    }
}
