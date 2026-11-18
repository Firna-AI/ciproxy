//! Public PostgreSQL metering integration tests.

pub(in crate::store) mod concurrent;
mod cursor_tests;
mod facts_tests;
mod observation_tests;
mod registration_tests;
mod removal_tests;
mod retention_tests;
mod support;
