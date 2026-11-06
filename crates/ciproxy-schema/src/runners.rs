//! Runner lifecycle and immutable resource placement.

diesel::table! {
    runners (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        organization_id -> Uuid,
        label_id -> Uuid,
        scale_set_id -> Uuid,
        github_id -> Nullable<Int8>,
        state -> Text,
        vcpus -> Int4,
        memory_bytes -> Int8,
        disk_bytes -> Int8,
        machine_id -> Nullable<Uuid>,
        first_slot -> Nullable<Int4>,
        boat_key_id -> Nullable<Uuid>,
        sandbox_id -> Nullable<Text>,
        boat_size_name -> Nullable<Text>,
    }
}
