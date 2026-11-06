//! Diesel query implementations of the hosted store interfaces.

#![warn(unreachable_pub)]

mod billing;
mod boat;
mod connection;
mod dashboard;
mod error;
mod exact_size;
mod gateway;
mod gateway_contacts;
mod gateway_registration;
mod gateway_transaction;
mod github;
mod installation_links;
mod jobs;
mod labels;
mod machine_observation;
mod machine_reads;
mod machine_removal;
mod machine_rows;
mod machines;
mod metering;
mod organization_lifecycle;
mod organization_rows;
mod organizations;
mod overflow;
mod own_lease;
mod own_observation;
mod own_reservation;
mod own_runners;
mod routing_snapshot;
mod runners;
mod scale_set_tracking;
mod scale_sets;
mod settings;
mod tokens;

pub mod factory;

mod workspace_credentials;
mod workspaces;
