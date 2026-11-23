DROP INDEX overflow_provider_accounts_active;
CREATE UNIQUE INDEX overflow_provider_accounts_active
    ON overflow_provider_accounts (workspace_id, provider)
    WHERE retired_at IS NULL;

CREATE TABLE overflow_provider_settings (
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    provider TEXT NOT NULL CHECK (provider IN ('boat')),
    spend_limit_usd_micros BIGINT CHECK (spend_limit_usd_micros > 0),
    offered_position INTEGER CHECK (offered_position >= 0),
    PRIMARY KEY (workspace_id, provider),
    UNIQUE (workspace_id, offered_position)
);

INSERT INTO overflow_provider_settings
    (workspace_id, provider, spend_limit_usd_micros, offered_position)
SELECT w.id, 'boat', s.boat_spend_limit_usd_micros, 0
FROM workspaces w
LEFT JOIN settings s ON s.workspace_id = w.id;

ALTER TABLE settings DROP COLUMN boat_spend_limit_usd_micros;

CREATE TABLE label_places (
    label_id UUID NOT NULL REFERENCES labels(id) ON DELETE CASCADE,
    position INTEGER NOT NULL CHECK (position >= 0),
    place TEXT NOT NULL CHECK (place IN ('machine', 'boat')),
    PRIMARY KEY (label_id, position),
    UNIQUE (label_id, place)
);

INSERT INTO label_places (label_id, position, place)
SELECT id, 0, 'machine' FROM labels WHERE allow_machines
UNION ALL
SELECT id, CASE WHEN allow_machines THEN 1 ELSE 0 END, 'boat'
FROM labels WHERE allow_boat;

ALTER TABLE labels DROP COLUMN allow_machines, DROP COLUMN allow_boat;

ALTER TABLE labels DROP CONSTRAINT labels_when_nothing_fits_check;
UPDATE labels SET when_nothing_fits = 'largest_overflow'
WHERE when_nothing_fits = 'largest_boat';
ALTER TABLE labels ADD CONSTRAINT labels_when_nothing_fits_check
    CHECK (when_nothing_fits IN ('wait', 'largest_overflow'));
