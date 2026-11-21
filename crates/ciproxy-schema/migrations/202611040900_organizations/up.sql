CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    github_id BIGINT NOT NULL,
    login TEXT NOT NULL,
    installation_id BIGINT NOT NULL UNIQUE,
    app_state TEXT NOT NULL DEFAULT 'active'
        CHECK (app_state IN ('active', 'suspended', 'removed')),
    repository_mode TEXT NOT NULL DEFAULT 'all_private'
        CHECK (repository_mode IN ('all_private', 'selected')),
    allowed_repository_ids BIGINT[] NOT NULL DEFAULT '{}',
    removed_at TIMESTAMPTZ,
    delete_after TIMESTAMPTZ,
    UNIQUE (workspace_id, id)
);
CREATE INDEX organizations_retention ON organizations (delete_after)
    WHERE app_state = 'removed';

CREATE UNIQUE INDEX organizations_current_github_id ON organizations (github_id)
    WHERE app_state <> 'removed';
