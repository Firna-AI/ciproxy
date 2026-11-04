CREATE TABLE boat_accounts (
    id UUID PRIMARY KEY,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    account_name TEXT NOT NULL,
    key_ciphertext BYTEA NOT NULL,
    connected_at TIMESTAMPTZ NOT NULL,
    retired_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    UNIQUE (organization_id, id)
);
CREATE UNIQUE INDEX boat_accounts_active ON boat_accounts (organization_id)
    WHERE retired_at IS NULL;
