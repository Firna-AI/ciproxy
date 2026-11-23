UPDATE metering_job_cursors
SET facts = jsonb_set(facts, '{place}', '"boat"')
WHERE facts -> 'place' = '{"overflow":{"provider":"boat"}}'::jsonb;

UPDATE overflow_runner_intents
SET body = body - 'provider' WHERE body ? 'provider';
UPDATE overflow_starts
SET record_json = (record_json::jsonb - 'provider')::text
WHERE record_json::jsonb ? 'provider';
UPDATE overflow_accounts
SET state_json = (state_json::jsonb - 'provider')::text
WHERE state_json::jsonb ? 'provider';

ALTER TABLE metering_daily_sizes RENAME COLUMN overflow_minutes TO boat_minutes;
ALTER TABLE metering_daily_sizes RENAME COLUMN overflow_starts TO boat_starts;
ALTER TABLE metering_overflow_cursors DROP COLUMN provider;
ALTER TABLE metering_daily_overflow_costs DROP COLUMN provider;
ALTER TABLE metering_overflow_costs DROP COLUMN provider;

ALTER INDEX metering_overflow_cursors_workspace RENAME TO metering_boat_cursors_workspace;
ALTER INDEX metering_overflow_costs_workspace RENAME TO metering_boat_costs_workspace;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_overflow_cursors_workspace_id_runner_id_fkey
    TO metering_boat_cursors_workspace_id_runner_id_fkey;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_overflow_cursors_workspace_id_fkey
    TO metering_boat_cursors_workspace_id_fkey;
ALTER TABLE metering_overflow_cursors
    RENAME CONSTRAINT metering_overflow_cursors_pkey TO metering_boat_cursors_pkey;
ALTER TABLE metering_daily_overflow_costs
    RENAME CONSTRAINT metering_daily_overflow_costs_workspace_id_fkey
    TO metering_daily_boat_costs_workspace_id_fkey;
ALTER TABLE metering_daily_overflow_costs
    RENAME CONSTRAINT metering_daily_overflow_costs_pkey TO metering_daily_boat_costs_pkey;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_overflow_costs_workspace_id_runner_id_fkey
    TO metering_boat_costs_workspace_id_runner_id_fkey;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_overflow_costs_workspace_id_fkey
    TO metering_boat_costs_workspace_id_fkey;
ALTER TABLE metering_overflow_costs
    RENAME CONSTRAINT metering_overflow_costs_pkey TO metering_boat_costs_pkey;
ALTER TABLE metering_overflow_cursors RENAME TO metering_boat_cursors;
ALTER TABLE metering_daily_overflow_costs RENAME TO metering_daily_boat_costs;
ALTER TABLE metering_overflow_costs RENAME TO metering_boat_costs;

ALTER INDEX runners_overflow_key RENAME TO runners_boat_key;
ALTER TABLE runners
    RENAME CONSTRAINT runners_workspace_id_overflow_key_id_fkey
    TO runners_workspace_id_boat_key_id_fkey;
ALTER TABLE runners RENAME COLUMN overflow_size_name TO boat_size_name;
ALTER TABLE runners RENAME COLUMN overflow_key_id TO boat_key_id;

ALTER VIEW overflow_registration_ownership RENAME TO boat_registration_ownership;
ALTER INDEX overflow_runner_intents_workspace RENAME TO boat_runner_intents_workspace;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT overflow_runner_intents_workspace_id_fkey
    TO boat_runner_intents_workspace_id_fkey;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT overflow_runner_intents_delivery_id_key
    TO boat_runner_intents_delivery_id_key;
ALTER TABLE overflow_runner_intents
    RENAME CONSTRAINT overflow_runner_intents_pkey TO boat_runner_intents_pkey;
ALTER TABLE overflow_runner_intents RENAME TO boat_runner_intents;

ALTER INDEX overflow_provider_accounts_active RENAME TO boat_accounts_active;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT overflow_provider_accounts_workspace_id_fkey
    TO boat_accounts_workspace_id_fkey;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT overflow_provider_accounts_workspace_id_id_key
    TO boat_accounts_workspace_id_id_key;
ALTER TABLE overflow_provider_accounts
    RENAME CONSTRAINT overflow_provider_accounts_pkey TO boat_accounts_pkey;
ALTER TABLE overflow_provider_accounts DROP COLUMN provider;
ALTER TABLE overflow_provider_accounts RENAME TO boat_accounts;
