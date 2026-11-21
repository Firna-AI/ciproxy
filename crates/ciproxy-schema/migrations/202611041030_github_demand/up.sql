CREATE TABLE github_initial_admin (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    github_user_id BIGINT NOT NULL
);

CREATE TABLE github_repositories (
    workspace_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    github_id BIGINT NOT NULL,
    full_name TEXT NOT NULL,
    private BOOLEAN NOT NULL,
    default_branch TEXT NOT NULL,
    PRIMARY KEY (organization_id, github_id),
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE
);

CREATE TABLE github_jobs (
    workspace_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    job_id UUID NOT NULL,
    scale_set_id UUID NOT NULL,
    label_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ,
    assignment_order TEXT NOT NULL,
    phase SMALLINT NOT NULL,
    completed_observed_at TIMESTAMPTZ,
    body JSONB NOT NULL,
    revision BIGINT NOT NULL DEFAULT 1,
    delivered_revision BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (organization_id, job_id),
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, organization_id, scale_set_id, label_id)
        REFERENCES scale_sets(workspace_id, organization_id, id, label_id)
);

CREATE INDEX github_jobs_pending ON github_jobs
    (workspace_id, organization_id, scale_set_id)
    WHERE delivered_revision < revision;
