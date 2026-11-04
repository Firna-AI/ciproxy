//! Machine registrations and their latest disk reports.

diesel::table! {
    machines (id) {
        id -> Uuid,
        organization_id -> Uuid,
        hostname -> Text,
        processor -> Text,
        hardware_threads -> Int4,
        memory_bytes -> Int8,
        kvm_available -> Bool,
        slots -> Int4,
        warm_runners -> Int4,
        draining -> Bool,
        last_contact_at -> Nullable<Timestamptz>,
        agent_version -> Text,
        removed_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_disks (organization_id, machine_id, path) {
        organization_id -> Uuid,
        machine_id -> Uuid,
        path -> Text,
        total_bytes -> Int8,
        free_bytes -> Int8,
    }
}
