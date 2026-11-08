//! Public service scenarios, sharing only infrastructure fixtures.

mod addons_tests;
mod changes_tests;
mod checked_stripe;
mod checkout_tests;
mod collection_decision_tests;
mod collection_exclusion_tests;
mod collection_stop_tests;
mod database;
mod expiry_recovery_tests;
mod expiry_support;
mod fake;
#[expect(
    dead_code,
    reason = "Credit fixture constructors are exercised by hosted control-plane tests"
)]
mod fake_credits;
mod fake_payments;
mod fake_requests;
mod fake_routes;
#[expect(
    dead_code,
    reason = "Credit request expectations are exercised by hosted control-plane tests"
)]
mod fake_subscriptions;
mod fixture;
mod http_tests;
mod live_replay;
mod live_replay_tests;
mod locked_observation_tests;
mod ownership_tests;
mod pending_context_tests;
mod quote_fixture;
mod removal_tests;
mod renewal_ownership_tests;
mod renewal_upgrade_tests;
mod states_tests;
mod subscription_requests;
mod terminal_tests;
