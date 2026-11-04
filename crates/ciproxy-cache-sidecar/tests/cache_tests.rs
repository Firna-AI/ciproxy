//! Public HTTP cache contracts against a no-secret GitHub fixture.

mod support;

#[path = "cases/fallback_tests.rs"]
mod fallback_tests;
#[path = "cases/scope_tests.rs"]
mod scope_tests;
#[path = "cases/storage_tests.rs"]
mod storage_tests;
#[path = "cases/wire_tests.rs"]
mod wire_tests;
