//! Default filesystem-boundary regressions for the production Docker adapter.

#![expect(
    dead_code,
    reason = "The integration harness imports production dependencies and their pure tests."
)]

#[path = "../src/boundaries.rs"]
mod boundaries;
#[path = "../src/budget_file.rs"]
mod budget_file;
#[path = "../src/connection_types.rs"]
mod connection_types;
#[path = "../src/docker.rs"]
mod docker;
#[path = "../src/docker_files.rs"]
mod docker_files;
#[path = "../src/docker_prepare.rs"]
mod docker_prepare;
#[path = "docker/mod.rs"]
mod docker_regressions;
#[path = "../src/error.rs"]
mod error;
#[path = "docker/fixtures.rs"]
mod fixtures;
#[path = "../src/host_system.rs"]
mod host_system;
#[path = "../src/image_artifact.rs"]
mod image_artifact;
#[path = "../src/_tests_/image_fixtures.rs"]
mod image_fixtures;
#[path = "../src/image_registry.rs"]
mod image_registry;
#[path = "../src/image_space.rs"]
mod image_space;
#[path = "../src/image_store.rs"]
mod image_store;
#[path = "../src/image_verification.rs"]
mod image_verification;
#[path = "../src/layout.rs"]
mod layout;
#[path = "../src/persistence.rs"]
mod persistence;
#[path = "../src/process.rs"]
mod process;
#[path = "../src/sparse_copy.rs"]
mod sparse_copy;
#[path = "../src/state.rs"]
mod state;
#[path = "../src/storage_sizing.rs"]
mod storage_sizing;
