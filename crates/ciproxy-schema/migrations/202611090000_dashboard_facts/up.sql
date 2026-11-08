ALTER TABLE dashboard_users ADD COLUMN name text;
ALTER TABLE dashboard_roles ADD COLUMN granted_at timestamptz;
ALTER TABLE dashboard_roles ADD COLUMN request_organizations jsonb;
ALTER TABLE organizations ADD COLUMN joined_at timestamptz;
ALTER TABLE organizations ALTER COLUMN joined_at SET DEFAULT CURRENT_TIMESTAMP;
ALTER TABLE organizations ADD COLUMN repositories_observed_at timestamptz;
