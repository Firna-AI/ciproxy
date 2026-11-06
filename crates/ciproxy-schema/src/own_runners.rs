//! Credential-free own-machine command delivery state.

diesel::table! {
    own_runner_commands (runner_id) {
        runner_id -> Uuid,
        workspace_id -> Uuid,
        label_name -> Text,
        command_id -> Uuid,
        issued_at -> Timestamptz,
        acknowledged -> Bool,
        slot_indices -> Array<Int8>,
        requested_size -> Jsonb,
    }
}

use crate::runners::runners;

diesel::allow_tables_to_appear_in_same_query!(own_runner_commands, runners,);
