//! Public store integration tests on the PostgreSQL at DATABASE_URL.

mod cleanup_tests;
mod connection_tests;
mod dashboard_continuation_tests;
mod dashboard_pending_tests;
mod gateway_support;
mod gateway_tests;
mod github;
mod install_revocation_tests;
mod isolation_tests;
mod jobs_tests;
mod key_erasure_tests;
mod label_order_migration_tests;
mod label_order_tests;
mod lifecycle_tests;
mod lock_tests;
mod machine_snapshot_tests;
mod metering;
mod overflow;
mod overflow_provider_accounts_tests;
mod overflow_provider_migration_tests;
mod overflow_provider_settings_tests;
mod platform_tests;
mod registration_tests;
mod retention_tests;
mod support;
mod tokens_tests;
