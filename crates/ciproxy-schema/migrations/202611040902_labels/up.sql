CREATE TABLE labels (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    vcpus INTEGER NOT NULL,
    memory_gb INTEGER NOT NULL,
    disk_gb INTEGER NOT NULL,
    allow_machines BOOLEAN NOT NULL DEFAULT TRUE,
    allow_boat BOOLEAN NOT NULL DEFAULT TRUE,
    when_nothing_fits TEXT NOT NULL DEFAULT 'wait'
        CHECK (when_nothing_fits IN ('wait', 'largest_boat')),
    UNIQUE (organization_id, id),
    UNIQUE (organization_id, name)
);
