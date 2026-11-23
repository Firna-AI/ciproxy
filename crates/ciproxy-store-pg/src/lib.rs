//! Diesel query implementations of the hosted store interfaces.

#![warn(unreachable_pub)]

mod billing;
mod boat;
mod boat_registration_ownership;
mod boat_results;
mod boat_runtime;
mod connection;
mod connector;
mod dashboard;
mod error;
mod exact_size;
mod gateway;
mod gateway_contacts;
mod gateway_registration;
mod gateway_transaction;
mod github;
mod installation_links;
mod job_views;
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
mod own_job_bindings;
mod own_lease;
mod own_observation;
mod own_reservation;
mod own_result_guard;
mod own_results;
mod own_runners;
mod platform_text;
mod routing_snapshot;
mod runners;
mod scale_set_tracking;
mod scale_sets;
mod settings;
mod tokens;

pub mod factory;

mod workspace_credentials;
mod workspaces;
