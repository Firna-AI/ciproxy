CREATE TABLE dashboard_users (
    id UUID PRIMARY KEY,
    github_id BIGINT NOT NULL UNIQUE,
    login TEXT NOT NULL,
    avatar_url TEXT NOT NULL,
    credentials BYTEA NOT NULL,
    checked_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE dashboard_memberships (
    user_id UUID NOT NULL REFERENCES dashboard_users(id) ON DELETE CASCADE,
    github_id BIGINT NOT NULL,
    owner BOOLEAN NOT NULL,
    PRIMARY KEY (user_id, github_id)
);
CREATE TABLE dashboard_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES dashboard_users(id) ON DELETE CASCADE,
    digest BYTEA NOT NULL UNIQUE,
    csrf_digest BYTEA NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE dashboard_login_states (
    id UUID PRIMARY KEY,
    digest BYTEA NOT NULL UNIQUE,
    verifier BYTEA NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE dashboard_roles (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    github_user_id BIGINT NOT NULL,
    role TEXT,
    requested_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (workspace_id, github_user_id)
);
CREATE TABLE dashboard_install_intents (
    requester_id BIGINT NOT NULL,
    github_organization_id BIGINT NOT NULL,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    PRIMARY KEY (requester_id, github_organization_id)
);
CREATE TABLE dashboard_takeouts (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    installation_id BIGINT NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE
);
ALTER TABLE labels ADD COLUMN deleted_at TIMESTAMPTZ;
ALTER TABLE labels DROP CONSTRAINT labels_workspace_id_name_key;
CREATE UNIQUE INDEX labels_active_workspace_name ON labels (workspace_id, name)
    WHERE deleted_at IS NULL;
CREATE TABLE dashboard_initial_admin_receipts (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE
);
