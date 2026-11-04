//! Typed persistence boundaries for the hosted ciproxy control plane.

#![warn(unreachable_pub)]

pub mod boat;
pub mod error;
pub mod gateway;
pub mod github;
pub mod github_lifecycle;
pub mod jobs;
pub mod labels;
pub mod machines;
pub mod organizations;
pub mod runners;
pub mod scale_sets;
pub mod secrets;
pub mod settings;
pub mod store;
pub mod tokens;
pub mod vm_size;

pub mod workspaces;
