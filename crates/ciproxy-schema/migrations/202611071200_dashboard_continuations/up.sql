CREATE TABLE dashboard_continuations (
    session_id UUID PRIMARY KEY,
    claim_ids UUID[] NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);
