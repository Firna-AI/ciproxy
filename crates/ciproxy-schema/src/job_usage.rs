//! Workload samples and ordered steps for a job runner.

diesel::table! {
    job_usage_samples (workspace_id, runner_id) {
        workspace_id -> Uuid,
        runner_id -> Uuid,
        platform -> Text,
        first_point_at_ms -> Int8,
        point_seconds -> Int8,
        cpu_milli_vcpu -> Array<Int8>,
        peak_memory_mib -> Array<Int8>,
        oom_kills -> Nullable<Array<Int8>>,
        is_final -> Bool,
        received_at -> Timestamptz,
    }
}

diesel::table! {
    job_usage_steps (workspace_id, runner_id, position) {
        workspace_id -> Uuid,
        runner_id -> Uuid,
        position -> Int4,
        name -> Text,
        started_at_ms -> Nullable<Int8>,
        ended_at_ms -> Nullable<Int8>,
        skipped -> Bool,
    }
}
