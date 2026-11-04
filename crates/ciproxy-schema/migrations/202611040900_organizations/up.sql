CREATE TABLE organizations (
    id UUID PRIMARY KEY,
    github_id BIGINT NOT NULL UNIQUE,
    login TEXT NOT NULL,
    installation_id BIGINT NOT NULL UNIQUE,
    app_state TEXT NOT NULL DEFAULT 'active'
        CHECK (app_state IN ('active', 'suspended', 'removed')),
    removed_at TIMESTAMPTZ,
    delete_after TIMESTAMPTZ
);
CREATE INDEX organizations_retention ON organizations (delete_after)
    WHERE app_state = 'removed';
