DROP TABLE installation_workspace_offers;
DROP TABLE installation_claims;
DROP TABLE installation_attempts;
CREATE TABLE dashboard_install_intents (
    requester_id BIGINT NOT NULL,
    github_organization_id BIGINT NOT NULL,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    PRIMARY KEY (requester_id, github_organization_id)
);
