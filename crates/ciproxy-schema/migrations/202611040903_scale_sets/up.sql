CREATE TABLE scale_sets (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    label_id UUID NOT NULL,
    github_id BIGINT NOT NULL,
    runner_group_id BIGINT NOT NULL,
    lock_key BIGSERIAL NOT NULL UNIQUE,
    UNIQUE (organization_id, id),
    UNIQUE (organization_id, id, label_id),
    UNIQUE (organization_id, label_id),
    UNIQUE (organization_id, github_id),
    FOREIGN KEY (organization_id, label_id)
        REFERENCES labels(organization_id, id)
);
