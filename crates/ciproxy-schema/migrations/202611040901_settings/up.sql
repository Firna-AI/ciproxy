CREATE TABLE settings (
    workspace_id UUID PRIMARY KEY REFERENCES workspaces(id) ON DELETE CASCADE,
    overflow_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    overflow_wait TEXT NOT NULL DEFAULT 'seconds30'
        CHECK (overflow_wait IN ('at_once', 'seconds15', 'seconds30', 'seconds60', 'seconds120')),
    boat_spend_limit_usd_micros BIGINT
        CHECK (boat_spend_limit_usd_micros > 0)
);
