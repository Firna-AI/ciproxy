//! Shared PostgreSQL schema and embedded migrations for hosted stores.

#![warn(unreachable_pub)]

pub mod billing;
pub mod boat;
pub mod dashboard;
pub mod github;
pub mod github_attempts;
pub mod jobs;
pub mod labels;
pub mod machines;
pub mod metering;
pub mod migrations;
pub mod organizations;
pub mod overflow;
pub mod own_runners;
pub mod runners;
pub mod scale_sets;
pub mod settings;
pub mod tokens;

pub mod workspaces;
