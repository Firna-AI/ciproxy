//! Routing settings isolated by organization.

diesel::table! {
    settings (organization_id) {
        organization_id -> Uuid,
        overflow_enabled -> Bool,
        overflow_wait -> Text,
        repository_mode -> Text,
        allowed_repository_ids -> Array<Int8>,
        boat_spend_limit_usd_micros -> Nullable<Int8>,
    }
}
