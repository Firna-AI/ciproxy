//! Typed persistence boundaries for the hosted ciproxy control plane.

#![warn(unreachable_pub)]

pub mod billing;
pub mod dashboard;
pub mod error;
pub mod gateway;
pub mod github;
pub mod github_lifecycle;
pub mod github_timing;
pub mod installation_association;
pub mod installation_links;
pub mod installation_results;
pub mod job_bindings;
pub mod job_views;
pub mod jobs;
pub mod labels;
pub mod machines;
pub mod metering;
pub mod organizations;
pub mod overflow;
pub mod overflow_delivery;
pub mod overflow_provider_accounts;
pub mod overflow_runtime;
pub mod own_runners;
pub mod runners;
pub mod runtime_results;
pub mod scale_sets;
pub mod secrets;
pub mod settings;
pub mod store;
pub mod tokens;
pub mod vm_size;

pub mod workspaces;
