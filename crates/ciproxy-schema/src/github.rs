//! GitHub repository snapshots and UUID job-demand notifications.

diesel::table! {
    github_initial_admin (workspace_id) {
        workspace_id -> Uuid,
        github_user_id -> Int8,
    }
}

diesel::table! {
    github_repositories (organization_id, github_id) {
        workspace_id -> Uuid,
        organization_id -> Uuid,
        github_id -> Int8,
        full_name -> Text,
        private -> Bool,
        default_branch -> Text,
    }
}

diesel::table! {
    github_jobs (organization_id, job_id) {
        workspace_id -> Uuid,
        organization_id -> Uuid,
        job_id -> Uuid,
        scale_set_id -> Uuid,
        label_id -> Uuid,
        assigned_at -> Nullable<Timestamptz>,
        assignment_order -> Text,
        phase -> Int2,
        completed_observed_at -> Nullable<Timestamptz>,
        body -> Jsonb,
        revision -> Int8,
        delivered_revision -> Int8,
    }
}
