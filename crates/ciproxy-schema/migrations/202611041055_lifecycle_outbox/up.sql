CREATE TABLE github_lifecycle_outbox (
    sequence BIGSERIAL PRIMARY KEY,
    github_id BIGINT NOT NULL,
    event JSONB NOT NULL
);

CREATE INDEX github_lifecycle_pending
    ON github_lifecycle_outbox (github_id, sequence);
