//! Workflow labels and requested VM sizes.

use crate::scale_sets::scale_sets;

diesel::table! {
    labels (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        name -> Text,
        platform -> Text,
        vcpus -> Int4,
        memory_gb -> Int4,
        disk_gb -> Int4,
        when_nothing_fits -> Text,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    label_places (label_id, position) {
        label_id -> Uuid,
        position -> Int4,
        place -> Text,
    }
}

diesel::joinable!(label_places -> labels (label_id));
diesel::allow_tables_to_appear_in_same_query!(labels, label_places);
diesel::allow_tables_to_appear_in_same_query!(labels, scale_sets);
