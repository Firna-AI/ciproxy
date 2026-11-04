CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    github_id BIGINT NOT NULL,
    label_id UUID NOT NULL,
    scale_set_id UUID NOT NULL,
    runner_id UUID,
    name TEXT NOT NULL,
    workflow TEXT NOT NULL,
    repository TEXT NOT NULL,
    git_ref TEXT,
    state TEXT NOT NULL CHECK (state IN ('queued', 'running', 'completed')),
    result TEXT CHECK (result IN (
        'success', 'failure', 'cancelled', 'skipped', 'timed_out',
        'action_required', 'neutral', 'stale', 'startup_failure'
    )),
    queued_at TIMESTAMPTZ NOT NULL,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    routing_reason TEXT,
    cache_source TEXT CHECK (cache_source IN ('machine', 'github', 'machine_and_github')),
    cache_restores BIGINT,
    cache_restored_bytes BIGINT,
    UNIQUE (organization_id, github_id),
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, organization_id, scale_set_id, label_id)
        REFERENCES scale_sets(workspace_id, organization_id, id, label_id),
    FOREIGN KEY (workspace_id, organization_id, runner_id, label_id, scale_set_id)
        REFERENCES runners(workspace_id, organization_id, id, label_id, scale_set_id),
    CHECK (
        (cache_source IS NOT NULL AND cache_source IN ('machine', 'machine_and_github'))
        OR (cache_restores IS NULL AND cache_restored_bytes IS NULL)
    )
);
CREATE INDEX jobs_retention ON jobs (completed_at) WHERE state = 'completed';
CREATE INDEX jobs_organization_queue ON jobs (organization_id, queued_at);
