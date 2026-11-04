//! Encrypted Boat key versions and safe account metadata.

diesel::table! {
    boat_accounts (id) {
        id -> Uuid,
        organization_id -> Uuid,
        account_name -> Text,
        key_ciphertext -> Bytea,
        connected_at -> Timestamptz,
        retired_at -> Nullable<Timestamptz>,
        rejected_at -> Nullable<Timestamptz>,
    }
}
