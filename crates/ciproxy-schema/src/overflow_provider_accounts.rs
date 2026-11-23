//! Encrypted overflow provider keys and safe account metadata.

diesel::table! {
    overflow_provider_accounts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        account_name -> Text,
        key_ciphertext -> Bytea,
        connected_at -> Timestamptz,
        retired_at -> Nullable<Timestamptz>,
        rejected_at -> Nullable<Timestamptz>,
        provider -> Text,
    }
}
