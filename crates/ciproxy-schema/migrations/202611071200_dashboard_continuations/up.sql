CREATE TABLE dashboard_continuations (
    session_id UUID PRIMARY KEY,
    claim_ids UUID[] NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE installation_pending_results (
    claim_id UUID NOT NULL REFERENCES installation_claims(id) ON DELETE CASCADE,
    requester_id BIGINT NOT NULL,
    revision BIGSERIAL NOT NULL,
    PRIMARY KEY (claim_id, requester_id)
);

CREATE INDEX installation_pending_results_requester
    ON installation_pending_results (requester_id);
