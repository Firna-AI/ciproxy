CREATE TABLE billing_payments (
    command_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    state_json TEXT NOT NULL,
    pending BOOLEAN NOT NULL
);

CREATE INDEX billing_payments_pending ON billing_payments (workspace_id) WHERE pending;
