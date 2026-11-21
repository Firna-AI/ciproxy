CREATE TABLE runners (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    label_id UUID NOT NULL,
    scale_set_id UUID NOT NULL,
    github_id BIGINT,
    state TEXT NOT NULL CHECK (state IN ('starting', 'ready', 'busy', 'stopping', 'stopped')),
    vcpus INTEGER NOT NULL,
    memory_gb INTEGER NOT NULL,
    disk_gb INTEGER NOT NULL,
    machine_id UUID,
    first_slot INTEGER,
    boat_key_id UUID,
    sandbox_id TEXT,
    boat_size_name TEXT,
    UNIQUE (workspace_id, organization_id, id),
    UNIQUE (workspace_id, organization_id, id, label_id, scale_set_id),
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, organization_id, scale_set_id, label_id)
        REFERENCES scale_sets(workspace_id, organization_id, id, label_id),
    FOREIGN KEY (workspace_id, machine_id)
        REFERENCES machines(workspace_id, id),
    FOREIGN KEY (workspace_id, boat_key_id)
        REFERENCES boat_accounts(workspace_id, id) ON DELETE SET NULL (boat_key_id),
    CHECK (
        (machine_id IS NOT NULL AND first_slot IS NOT NULL
            AND boat_key_id IS NULL AND sandbox_id IS NULL AND boat_size_name IS NULL)
        OR
        (machine_id IS NULL AND first_slot IS NULL
            AND sandbox_id IS NOT NULL AND boat_size_name IS NOT NULL)
    )
);
CREATE INDEX runners_boat_key ON runners (workspace_id, boat_key_id)
    WHERE state <> 'stopped';
