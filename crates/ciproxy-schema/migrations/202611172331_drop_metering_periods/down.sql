CREATE TABLE metering_periods (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    starts_at TIMESTAMPTZ NOT NULL,
    ends_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (workspace_id, starts_at)
);
