//! Diesel query implementations of the hosted store interfaces.

#![warn(unreachable_pub)]

mod boat;
mod connection;
mod error;
mod jobs;
mod labels;
mod machine_removal;
mod machines;
mod metering;
mod organization_lifecycle;
mod organization_rows;
mod organizations;
mod runners;
mod scale_sets;
mod settings;
mod tokens;

pub mod factory;

mod workspace_credentials;
mod workspaces;
