//! Shared wire types for the ciproxy agent and control plane.

pub mod agent;
pub mod cache;
pub mod cache_metadata;
pub mod guest;
pub mod identifiers;
pub mod images;
pub mod jobs;
pub mod reports;
pub mod resources;
pub mod runners;
pub mod runtime;
pub mod secret;
pub mod version;

#[cfg(test)]
#[path = "_tests_/wire/mod.rs"]
mod wire_tests;
