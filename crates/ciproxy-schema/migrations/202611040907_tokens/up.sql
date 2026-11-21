CREATE TABLE tokens (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID,
    token_hash BYTEA NOT NULL UNIQUE,
    purpose TEXT NOT NULL CHECK (purpose IN ('install', 'machine', 'sandbox')),
    machine_id UUID,
    runner_id UUID,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    FOREIGN KEY (workspace_id, machine_id)
        REFERENCES machines(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, organization_id, runner_id)
        REFERENCES runners(workspace_id, organization_id, id) ON DELETE CASCADE,
    CHECK (
        (purpose = 'install' AND organization_id IS NULL AND machine_id IS NULL AND runner_id IS NULL
            AND expires_at IS NOT NULL AND expires_at = created_at + INTERVAL '1 hour')
        OR
        (purpose = 'machine' AND organization_id IS NULL AND machine_id IS NOT NULL AND runner_id IS NULL
            AND expires_at IS NULL)
        OR
        (purpose = 'sandbox' AND organization_id IS NOT NULL AND machine_id IS NULL AND runner_id IS NOT NULL)
    )
);
CREATE INDEX tokens_workspace ON tokens (workspace_id);
