CREATE TABLE installation_attempts (
    id UUID PRIMARY KEY,
    state_digest BYTEA NOT NULL UNIQUE,
    requester_id BIGINT NOT NULL,
    direct BOOLEAN NOT NULL,
    installation_id BIGINT
);
CREATE TABLE installation_claims (
    id UUID PRIMARY KEY,
    installation_id BIGINT NOT NULL UNIQUE,
    github_id BIGINT NOT NULL,
    login TEXT NOT NULL,
    requester_id BIGINT NOT NULL,
    event_seen BOOLEAN NOT NULL DEFAULT FALSE,
    user_verified BOOLEAN NOT NULL DEFAULT FALSE,
    direct BOOLEAN NOT NULL DEFAULT FALSE,
    workspace_id UUID REFERENCES workspaces(id) ON DELETE SET NULL,
    organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL,
    verified_at TIMESTAMPTZ,
    admin_approved BOOLEAN
);
CREATE TABLE installation_workspace_offers (
    claim_id UUID NOT NULL REFERENCES installation_claims(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    PRIMARY KEY (claim_id, workspace_id)
);
DROP TABLE dashboard_install_intents;
