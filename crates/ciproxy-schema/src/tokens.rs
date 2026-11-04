//! Hash-only credentials with explicit scopes, expiry, and revocation.

diesel::table! {
    tokens (id) {
        id -> Uuid,
        organization_id -> Uuid,
        token_hash -> Bytea,
        purpose -> Text,
        machine_id -> Nullable<Uuid>,
        runner_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        expires_at -> Nullable<Timestamptz>,
        revoked_at -> Nullable<Timestamptz>,
    }
}
