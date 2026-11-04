//! Machine registrations and their latest disk reports.

diesel::table! {
    machines (id) {
        id -> Uuid,
        workspace_id -> Uuid,
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
        registered_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    machine_disks (workspace_id, machine_id, path) {
        workspace_id -> Uuid,
        machine_id -> Uuid,
        path -> Text,
        total_bytes -> Int8,
        free_bytes -> Int8,
    }
}

diesel::table! {
    machine_reports (workspace_id, machine_id) {
        workspace_id -> Uuid,
        machine_id -> Uuid,
        session_id -> Uuid,
        report -> Jsonb,
    }
}
