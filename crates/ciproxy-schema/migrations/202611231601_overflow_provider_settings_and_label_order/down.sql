ALTER TABLE labels DROP CONSTRAINT labels_when_nothing_fits_check;
UPDATE labels SET when_nothing_fits = 'largest_boat'
WHERE when_nothing_fits = 'largest_overflow';
ALTER TABLE labels ADD CONSTRAINT labels_when_nothing_fits_check
    CHECK (when_nothing_fits IN ('wait', 'largest_boat'));

ALTER TABLE labels ADD COLUMN allow_machines BOOLEAN NOT NULL DEFAULT TRUE,
    ADD COLUMN allow_boat BOOLEAN NOT NULL DEFAULT TRUE;
UPDATE labels l SET
    allow_machines = EXISTS (
        SELECT 1 FROM label_places p WHERE p.label_id = l.id AND p.place = 'machine'
    ),
    allow_boat = EXISTS (
        SELECT 1 FROM label_places p WHERE p.label_id = l.id AND p.place = 'boat'
    );
DROP TABLE label_places;

ALTER TABLE settings ADD COLUMN boat_spend_limit_usd_micros BIGINT
    CHECK (boat_spend_limit_usd_micros > 0);
UPDATE settings s SET boat_spend_limit_usd_micros = p.spend_limit_usd_micros
FROM overflow_provider_settings p
WHERE p.workspace_id = s.workspace_id AND p.provider = 'boat';
DROP TABLE overflow_provider_settings;

DROP INDEX overflow_provider_accounts_active;
CREATE UNIQUE INDEX overflow_provider_accounts_active
    ON overflow_provider_accounts (workspace_id)
    WHERE retired_at IS NULL;
