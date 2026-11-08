//! Isolated real-store HTTP scenarios and controlled external services.

mod access_tests;
mod database;
mod fixture;
mod machine_tests;
mod ports;
mod ports_gate;
mod ports_scale_sets;
mod runtime_inputs;
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

mod setup_continuation_tests;
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

mod continuation_tests;
mod direct_recovery_tests;
mod late_approval_tests;
mod link_final_tests;
mod m18_access_tests;
mod m18_checkout_tests;
mod m18_fit_support;
mod m18_fit_tests;
mod m18_gap_support;
mod m18_job_requests_support;
mod m18_job_requests_tests;
mod m18_jobs_support;
mod m18_jobs_tests;
mod m18_live_tests;
mod m18_membership_support;
mod m18_membership_tests;
mod m18_mount_tests;
mod m18_overflow_tests;
mod m18_paid_tests;
mod m18_quote;
mod m18_savings_tests;
mod m18_server_tests;
mod m18_stripe;
mod m18_support;
mod m18_usage_tests;
#[path = "../../../ciproxy-control-plane/tests/runtime/overflow_ports.rs"]
mod overflow_ports;
mod overflow_spend_tests;
mod overflow_support;
mod overflow_tests;
mod overflow_wallet_tests;
mod pending_result_tests;
mod session_failure_tests;
mod signed_association_tests;
