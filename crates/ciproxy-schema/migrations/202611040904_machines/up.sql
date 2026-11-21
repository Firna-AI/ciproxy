CREATE TABLE machines (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    hostname TEXT NOT NULL,
    processor TEXT NOT NULL,
    hardware_threads INTEGER NOT NULL,
    memory_bytes BIGINT NOT NULL,
    kvm_available BOOLEAN NOT NULL,
    slots INTEGER NOT NULL,
    warm_runners INTEGER NOT NULL DEFAULT 2,
    draining BOOLEAN NOT NULL DEFAULT FALSE,
    last_contact_at TIMESTAMPTZ,
    agent_version TEXT NOT NULL,
    removed_at TIMESTAMPTZ,
    UNIQUE (workspace_id, id)
);
CREATE TABLE machine_disks (
    workspace_id UUID NOT NULL,
    machine_id UUID NOT NULL,
    path TEXT NOT NULL,
    total_bytes BIGINT NOT NULL,
    free_bytes BIGINT NOT NULL,
    PRIMARY KEY (workspace_id, machine_id, path),
    FOREIGN KEY (workspace_id, machine_id)
        REFERENCES machines(workspace_id, id) ON DELETE CASCADE
);
