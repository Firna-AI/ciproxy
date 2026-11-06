CREATE TABLE organization_activation (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    admitted BOOLEAN NOT NULL,
    billing_hold BOOLEAN NOT NULL,
    admin_approval_hold BOOLEAN NOT NULL,
    added_by BIGINT
);
INSERT INTO organization_activation
    (organization_id,workspace_id,admitted,billing_hold,admin_approval_hold)
SELECT id,workspace_id,TRUE,FALSE,FALSE FROM organizations;
