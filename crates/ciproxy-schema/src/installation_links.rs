//! Durable token-free installation association and independent admission metadata.

diesel::table! {
    installation_attempts (id) {
        id -> Uuid,
        state_digest -> Bytea,
        requester_id -> Int8,
        direct -> Bool,
        installation_id -> Nullable<Int8>,
    }
}

diesel::table! {
    installation_claims (id) {
        id -> Uuid,
        installation_id -> Int8,
        github_id -> Int8,
        login -> Text,
        requester_id -> Int8,
        event_seen -> Bool,
        user_verified -> Bool,
        direct -> Bool,
        workspace_id -> Nullable<Uuid>,
        organization_id -> Nullable<Uuid>,
        verified_at -> Nullable<Timestamptz>,
        admin_approved -> Nullable<Bool>,
    }
}

diesel::table! {
    installation_workspace_offers (claim_id, workspace_id) {
        claim_id -> Uuid,
        workspace_id -> Uuid,
    }
}

diesel::table! {
    organization_activation (organization_id) {
        organization_id -> Uuid,
        workspace_id -> Uuid,
        admitted -> Bool,
        billing_hold -> Bool,
        admin_approval_hold -> Bool,
        added_by -> Nullable<Int8>,
    }
}
