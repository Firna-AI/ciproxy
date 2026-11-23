ALTER TABLE boat_accounts RENAME TO overflow_provider_accounts;
ALTER TABLE overflow_provider_accounts ADD COLUMN provider TEXT NOT NULL DEFAULT 'boat'
    CHECK (provider IN ('boat'));
ALTER TABLE overflow_provider_accounts ALTER COLUMN provider DROP DEFAULT;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT boat_accounts_pkey TO overflow_provider_accounts_pkey;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT boat_accounts_workspace_id_id_key
    TO overflow_provider_accounts_workspace_id_id_key;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT boat_accounts_workspace_id_fkey
    TO overflow_provider_accounts_workspace_id_fkey;
ALTER INDEX boat_accounts_active RENAME TO overflow_provider_accounts_active;

ALTER TABLE boat_runner_intents RENAME TO overflow_runner_intents;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT boat_runner_intents_pkey TO overflow_runner_intents_pkey;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT boat_runner_intents_delivery_id_key
    TO overflow_runner_intents_delivery_id_key;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT boat_runner_intents_workspace_id_fkey
    TO overflow_runner_intents_workspace_id_fkey;
ALTER INDEX boat_runner_intents_workspace RENAME TO overflow_runner_intents_workspace;
ALTER VIEW boat_registration_ownership RENAME TO overflow_registration_ownership;

ALTER TABLE runners RENAME COLUMN boat_key_id TO overflow_key_id;
ALTER TABLE runners RENAME COLUMN boat_size_name TO overflow_size_name;
ALTER TABLE runners
    RENAME CONSTRAINT runners_workspace_id_boat_key_id_fkey
    TO runners_workspace_id_overflow_key_id_fkey;
ALTER INDEX runners_boat_key RENAME TO runners_overflow_key;

ALTER TABLE metering_boat_costs RENAME TO metering_overflow_costs;
ALTER TABLE metering_daily_boat_costs RENAME TO metering_daily_overflow_costs;
ALTER TABLE metering_boat_cursors RENAME TO metering_overflow_cursors;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_boat_costs_pkey TO metering_overflow_costs_pkey;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_boat_costs_workspace_id_fkey
    TO metering_overflow_costs_workspace_id_fkey;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_boat_costs_workspace_id_runner_id_fkey
    TO metering_overflow_costs_workspace_id_runner_id_fkey;
ALTER TABLE metering_daily_overflow_costs
    RENAME CONSTRAINT metering_daily_boat_costs_pkey TO metering_daily_overflow_costs_pkey;
ALTER TABLE metering_daily_overflow_costs
    RENAME CONSTRAINT metering_daily_boat_costs_workspace_id_fkey
    TO metering_daily_overflow_costs_workspace_id_fkey;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_boat_cursors_pkey TO metering_overflow_cursors_pkey;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_boat_cursors_workspace_id_fkey
    TO metering_overflow_cursors_workspace_id_fkey;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_boat_cursors_workspace_id_runner_id_fkey
    TO metering_overflow_cursors_workspace_id_runner_id_fkey;
ALTER INDEX metering_boat_costs_workspace RENAME TO metering_overflow_costs_workspace;
ALTER INDEX metering_boat_cursors_workspace RENAME TO metering_overflow_cursors_workspace;

ALTER TABLE metering_overflow_costs ADD COLUMN provider TEXT NOT NULL DEFAULT 'boat'
    CHECK (provider IN ('boat'));
ALTER TABLE metering_overflow_costs ALTER COLUMN provider DROP DEFAULT;
ALTER TABLE metering_daily_overflow_costs ADD COLUMN provider TEXT NOT NULL DEFAULT 'boat'
    CHECK (provider IN ('boat'));
ALTER TABLE metering_daily_overflow_costs ALTER COLUMN provider DROP DEFAULT;
ALTER TABLE metering_overflow_cursors ADD COLUMN provider TEXT NOT NULL DEFAULT 'boat'
    CHECK (provider IN ('boat'));
ALTER TABLE metering_overflow_cursors ALTER COLUMN provider DROP DEFAULT;
ALTER TABLE metering_daily_sizes RENAME COLUMN boat_starts TO overflow_starts;
ALTER TABLE metering_daily_sizes RENAME COLUMN boat_minutes TO overflow_minutes;

UPDATE overflow_accounts
SET state_json = (state_json::jsonb || '{"provider":"boat"}'::jsonb)::text
WHERE NOT state_json::jsonb ? 'provider';
UPDATE overflow_starts
SET record_json = (record_json::jsonb || '{"provider":"boat"}'::jsonb)::text
WHERE NOT record_json::jsonb ? 'provider';
UPDATE overflow_runner_intents
SET body = body || '{"provider":"boat"}'::jsonb
WHERE NOT body ? 'provider';

UPDATE metering_job_cursors
SET facts = jsonb_set(facts, '{place}', '{"overflow":{"provider":"boat"}}')
WHERE facts -> 'place' = '"boat"'::jsonb;
