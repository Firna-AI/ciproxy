//! Credential-free runner identities saved before Boat registration delivery.

diesel::table! {
    boat_runner_intents (runner_id) {
        runner_id -> Uuid,
        workspace_id -> Uuid,
        delivery_id -> Uuid,
        organization_id -> Uuid,
        label_id -> Uuid,
        scale_set_id -> Uuid,
        body -> Jsonb,
        installation_id -> Nullable<BigInt>,
        delete_after -> Nullable<Timestamptz>,
    }
}
