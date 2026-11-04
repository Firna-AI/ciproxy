//! Shared PostgreSQL schema and embedded migrations for hosted stores.

#![warn(unreachable_pub)]

pub mod billing;
pub mod boat;
pub mod jobs;
pub mod labels;
pub mod machines;
pub mod migrations;
pub mod organizations;
pub mod runners;
pub mod scale_sets;
pub mod settings;
pub mod tokens;

pub mod workspaces;
