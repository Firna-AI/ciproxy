//! Metering history, anonymous aggregates, and job-bound accounting cursors.

diesel::table! {
    metering_machine_costs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        machine_id -> Uuid,
        starts_at -> Timestamptz,
        ends_at -> Nullable<Timestamptz>,
        monthly_usd -> Numeric,
    }
}
diesel::table! {
    metering_plan_costs (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        starts_at -> Timestamptz,
        ends_at -> Nullable<Timestamptz>,
        monthly_usd -> Numeric,
    }
}
diesel::table! {
    metering_boat_costs (runner_id) {
        runner_id -> Uuid,
        workspace_id -> Uuid,
        starts_at -> Timestamptz,
        ends_at -> Nullable<Timestamptz>,
        start_usd -> Numeric,
        minute_usd -> Numeric,
    }
}
diesel::table! {
    metering_periods (workspace_id, starts_at) {
        workspace_id -> Uuid,
        starts_at -> Timestamptz,
        ends_at -> Timestamptz,
    }
}
diesel::table! {
    metering_daily_sizes (workspace_id, day, vcpus, memory_bytes, disk_bytes) {
        workspace_id -> Uuid,
        day -> Date,
        vcpus -> Int4,
        memory_bytes -> Int8,
        disk_bytes -> Int8,
        machine_starts -> Int8,
        machine_minutes -> Numeric,
        boat_starts -> Int8,
        boat_minutes -> Numeric,
    }
}
diesel::table! {
    metering_daily_boat_costs (workspace_id, day) {
        workspace_id -> Uuid,
        day -> Date,
        usd -> Numeric,
    }
}
diesel::table! {
    metering_start_totals (workspace_id, taken_at) {
        workspace_id -> Uuid,
        taken_at -> Text,
        starts -> Int8,
    }
}
diesel::table! {
    metering_job_cursors (workspace_id, organization_id, source, job_id) {
        job_id -> Uuid,
        workspace_id -> Uuid,
        organization_id -> Uuid,
        observed_at -> Text,
        source -> Text,
        facts -> Jsonb,
    }
}
diesel::table! {
    metering_boat_cursors (runner_id) {
        runner_id -> Uuid,
        workspace_id -> Uuid,
        starts_at -> Timestamptz,
        ends_at -> Nullable<Timestamptz>,
        start_usd -> Numeric,
        minute_usd -> Numeric,
        observed_at -> Timestamptz,
    }
}
