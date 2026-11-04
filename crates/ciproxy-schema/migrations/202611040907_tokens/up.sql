CREATE TABLE tokens (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    token_hash BYTEA NOT NULL UNIQUE,
    purpose TEXT NOT NULL CHECK (purpose IN ('install', 'machine', 'sandbox')),
    machine_id UUID,
    runner_id UUID,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    FOREIGN KEY (organization_id, machine_id)
        REFERENCES machines(organization_id, id) ON DELETE CASCADE,
    FOREIGN KEY (organization_id, runner_id)
        REFERENCES runners(organization_id, id) ON DELETE CASCADE,
    CHECK (
        (purpose = 'install' AND machine_id IS NULL AND runner_id IS NULL
            AND expires_at IS NOT NULL AND expires_at = created_at + INTERVAL '1 hour')
        OR
        (purpose = 'machine' AND machine_id IS NOT NULL AND runner_id IS NULL
            AND expires_at IS NULL)
        OR
        (purpose = 'sandbox' AND machine_id IS NULL AND runner_id IS NOT NULL)
    )
);
CREATE INDEX tokens_organization ON tokens (organization_id);
