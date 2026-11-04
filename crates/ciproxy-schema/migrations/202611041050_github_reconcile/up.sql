CREATE INDEX github_jobs_run_facts ON github_jobs
    (workspace_id, organization_id, ((body->>'run_id')::bigint));
