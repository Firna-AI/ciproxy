//! Share existing API and Stripe infrastructure without copying their behavior.

#![expect(
    dead_code,
    reason = "Shared HTTP fixtures expose additional scenario helpers"
)]

#[path = "../../../ciproxy-api/tests/http/database.rs"]
mod database;
#[path = "../../../ciproxy-billing/tests/_tests_/fake.rs"]
mod fake;
#[path = "../../../ciproxy-billing/tests/_tests_/fake_credits.rs"]
mod fake_credits;
#[path = "../../../ciproxy-billing/tests/_tests_/fake_payments.rs"]
mod fake_payments;
#[path = "../../../ciproxy-billing/tests/_tests_/fake_requests.rs"]
mod fake_requests;
#[path = "../../../ciproxy-billing/tests/_tests_/fake_routes.rs"]
mod fake_routes;
#[path = "../../../ciproxy-billing/tests/_tests_/fake_subscriptions.rs"]
mod fake_subscriptions;
#[path = "../../../ciproxy-api/tests/http/fixture.rs"]
mod fixture;
#[path = "../../../ciproxy-api/tests/http/fixture_io.rs"]
mod fixture_io;
#[path = "../../../ciproxy-api/tests/http/ports.rs"]
mod ports;
#[path = "../../../ciproxy-api/tests/http/ports_gate.rs"]
mod ports_gate;
#[path = "../../../ciproxy-api/tests/http/ports_scale_sets.rs"]
mod ports_scale_sets;
#[path = "../../../ciproxy-api/tests/http/runtime_inputs.rs"]
mod runtime_inputs;

mod cancellation_tests;
mod composition_tests;
mod credit_guard_tests;
mod credit_portal_tests;
mod credit_retry_tests;
mod credit_settlement_tests;
mod credit_test_support;
mod operator_invoice_tests;
mod operator_support;
mod operator_write_tests;
mod paid_quote_refusal_tests;
mod paid_quote_tests;
mod paid_review_tests;
mod pending_credit_tests;
mod preview_support;
mod quote_support;
mod support;
mod support_host;
