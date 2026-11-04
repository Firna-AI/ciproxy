CREATE SEQUENCE billing_lock_keys MINVALUE -9223372036854775808
    MAXVALUE -5000000000 START -9223372036854775808;

CREATE TABLE billing_accounts (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    lock_key BIGINT NOT NULL UNIQUE DEFAULT nextval('billing_lock_keys'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    state_json TEXT,
    command_json TEXT,
    checkout_json TEXT,
    subscription_id TEXT UNIQUE,
    removal_cancel_requested BOOLEAN NOT NULL DEFAULT FALSE
);

INSERT INTO billing_accounts (workspace_id) SELECT id FROM workspaces;

CREATE TABLE billing_events (
    event_id TEXT PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    applied_at TIMESTAMPTZ NOT NULL
);
