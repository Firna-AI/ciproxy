CREATE TABLE billing_quotes (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    quote_json TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    command_id UUID
);

CREATE INDEX billing_quotes_workspace_expiry ON billing_quotes (workspace_id, expires_at);
