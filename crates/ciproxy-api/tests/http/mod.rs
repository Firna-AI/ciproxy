//! Isolated real-store HTTP scenarios and controlled external services.

mod access_tests;
mod database;
mod fixture;
mod machine_tests;
mod ports;
mod ports_gate;
mod ports_scale_sets;
mod settings_tests;

mod fixture_io;
mod membership_tests;
mod organization_tests;
mod recovery_tests;

mod adapter_server;
mod adapter_tests;

mod retention_tests;

mod initial_admin_tests;

mod takeout_reinstall_tests;

mod capacity_fixture;

mod oauth_tests;

mod maintenance_tests;

mod setup_peer;
mod setup_support;
mod setup_tests;

mod idle_retirement_tests;
mod label_reactivation_tests;
mod label_retirement_support;
mod label_retirement_tests;
mod link_flow_tests;
mod link_guard_support;
mod link_hold_tests;
mod link_race_tests;
mod link_recovery_tests;
mod link_role_tests;
mod link_support;
mod oauth_peer;
mod refresh_tests;
mod token_tests;

mod setup_personal_tests;

mod link_final_tests;
