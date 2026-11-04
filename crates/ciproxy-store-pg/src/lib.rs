//! Diesel query implementations of the hosted store interfaces.

#![warn(unreachable_pub)]

mod boat;
mod connection;
mod error;
mod gateway;
mod gateway_contacts;
mod gateway_registration;
mod gateway_transaction;
mod github;
mod jobs;
mod labels;
mod machine_observation;
mod machine_reads;
mod machine_rows;
mod machines;
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
