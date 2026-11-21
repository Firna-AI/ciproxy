ALTER TABLE organizations DROP COLUMN repositories_observed_at;
ALTER TABLE organizations DROP COLUMN joined_at;
ALTER TABLE dashboard_roles DROP COLUMN request_organizations;
ALTER TABLE dashboard_roles DROP COLUMN granted_at;
ALTER TABLE dashboard_users DROP COLUMN name;
