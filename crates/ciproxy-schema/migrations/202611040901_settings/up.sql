CREATE TABLE settings (
    organization_id UUID PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    overflow_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    overflow_wait TEXT NOT NULL DEFAULT 'seconds30'
        CHECK (overflow_wait IN ('at_once', 'seconds15', 'seconds30', 'seconds60', 'seconds120')),
    repository_mode TEXT NOT NULL DEFAULT 'all_private'
        CHECK (repository_mode IN ('all_private', 'selected')),
    allowed_repository_ids BIGINT[] NOT NULL DEFAULT '{}',
    boat_spend_limit_usd_micros BIGINT
        CHECK (boat_spend_limit_usd_micros > 0)
);
