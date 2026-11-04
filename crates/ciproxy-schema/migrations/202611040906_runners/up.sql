CREATE TABLE runners (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
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
    UNIQUE (organization_id, id),
    UNIQUE (organization_id, id, label_id, scale_set_id),
    FOREIGN KEY (organization_id, scale_set_id, label_id)
        REFERENCES scale_sets(organization_id, id, label_id),
    FOREIGN KEY (organization_id, machine_id)
        REFERENCES machines(organization_id, id),
    FOREIGN KEY (organization_id, boat_key_id)
        REFERENCES boat_accounts(organization_id, id) ON DELETE SET NULL (boat_key_id),
    CHECK (
        (machine_id IS NOT NULL AND first_slot IS NOT NULL
            AND boat_key_id IS NULL AND sandbox_id IS NULL AND boat_size_name IS NULL)
        OR
        (machine_id IS NULL AND first_slot IS NULL
            AND sandbox_id IS NOT NULL AND boat_size_name IS NOT NULL)
    )
);
CREATE INDEX runners_boat_key ON runners (organization_id, boat_key_id)
    WHERE state <> 'stopped';
