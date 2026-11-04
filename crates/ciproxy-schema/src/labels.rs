//! Workflow labels and requested VM sizes.

diesel::table! {
    labels (id) {
        id -> Uuid,
        organization_id -> Uuid,
        name -> Text,
        vcpus -> Int4,
        memory_gb -> Int4,
        disk_gb -> Int4,
        allow_machines -> Bool,
        allow_boat -> Bool,
        when_nothing_fits -> Text,
    }
}
