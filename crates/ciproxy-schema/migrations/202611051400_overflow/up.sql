CREATE TABLE overflow_accounts (
    key_id UUID PRIMARY KEY REFERENCES boat_accounts(id) ON DELETE CASCADE,
    state_json TEXT NOT NULL
);

CREATE TABLE overflow_starts (
    delivery_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    runner_id UUID NOT NULL UNIQUE,
    key_id UUID,
    record_json TEXT NOT NULL,
    launch_ciphertext BYTEA,
    token_hash BYTEA NOT NULL UNIQUE,
    reported_at TIMESTAMPTZ,
    ends_at TIMESTAMPTZ,
    FOREIGN KEY (workspace_id, key_id)
        REFERENCES boat_accounts(workspace_id, id) ON DELETE SET NULL (key_id)
);
CREATE INDEX overflow_starts_workspace ON overflow_starts(workspace_id);
CREATE INDEX overflow_starts_live_key ON overflow_starts(workspace_id, key_id)
    WHERE ends_at IS NULL;

CREATE SEQUENCE overflow_lock_key_seq MINVALUE -4999999999
    MAXVALUE -1 START -1 INCREMENT -1;
CREATE TABLE overflow_locks (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    lock_key BIGINT NOT NULL UNIQUE DEFAULT nextval('overflow_lock_key_seq')
);
