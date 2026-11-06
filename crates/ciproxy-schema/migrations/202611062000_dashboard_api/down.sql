DROP TABLE dashboard_initial_admin_receipts;
DROP INDEX labels_active_workspace_name;
ALTER TABLE labels ADD CONSTRAINT labels_workspace_id_name_key
    UNIQUE (workspace_id, name);
ALTER TABLE labels DROP COLUMN deleted_at;
DROP TABLE dashboard_takeouts;
DROP TABLE dashboard_install_intents;
DROP TABLE dashboard_roles;
DROP TABLE dashboard_login_states;
DROP TABLE dashboard_sessions;
DROP TABLE dashboard_memberships;
DROP TABLE dashboard_users;
