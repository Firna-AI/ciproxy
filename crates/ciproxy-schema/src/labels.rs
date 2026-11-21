//! Workflow labels and requested VM sizes.

use crate::scale_sets::scale_sets;

diesel::table! {
    labels (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        vcpus -> Int4,
        memory_gb -> Int4,
        disk_gb -> Int4,
        allow_machines -> Bool,
        allow_boat -> Bool,
        when_nothing_fits -> Text,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(labels, scale_sets);
