CREATE TABLE own_runner_commands (
    runner_id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL,
    label_name TEXT NOT NULL,
    command_id UUID NOT NULL UNIQUE,
    issued_at TIMESTAMPTZ NOT NULL,
    acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
    slot_indices BIGINT[] NOT NULL DEFAULT '{}',
    requested_size JSONB NOT NULL,
    FOREIGN KEY (workspace_id, runner_id)
        REFERENCES runners (workspace_id, id) ON DELETE CASCADE
);

CREATE SEQUENCE own_machine_lock_keys MINVALUE -6999999999
    MAXVALUE -6000000000 START -6999999999;
CREATE TABLE own_machine_leases (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    lock_key BIGINT NOT NULL UNIQUE DEFAULT nextval('own_machine_lock_keys')
);
