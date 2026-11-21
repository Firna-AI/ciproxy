CREATE TABLE scale_sets (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL,
    label_id UUID NOT NULL,
    github_id BIGINT NOT NULL,
    runner_group_id BIGINT NOT NULL,
    lock_key BIGSERIAL NOT NULL UNIQUE,
    UNIQUE (workspace_id, organization_id, id),
    UNIQUE (workspace_id, organization_id, id, label_id),
    UNIQUE (organization_id, label_id),
    UNIQUE (organization_id, github_id),
    FOREIGN KEY (workspace_id, organization_id)
        REFERENCES organizations(workspace_id, id) ON DELETE CASCADE,
    FOREIGN KEY (workspace_id, label_id)
        REFERENCES labels(workspace_id, id)
);
