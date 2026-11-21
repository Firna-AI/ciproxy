//! Exact assignment-order identities for taken scale-set job attempts.

diesel::table! {
    github_job_attempts (id) {
        id -> Uuid,
        workspace_id -> Uuid,
        organization_id -> Uuid,
        job_id -> Uuid,
        assignment_order -> Text,
        phase -> Int2,
        completed_observed_at -> Nullable<Timestamptz>,
        body -> Jsonb,
    }
}
