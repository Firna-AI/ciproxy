//! Composition root of the open-source ciproxy machine agent.

mod application;
mod boundaries;
mod cache;
mod capacity;
mod cli;
mod commands;
mod connection;
mod controller;
mod error;
mod gateway;
mod hardware;
mod image_artifact;
mod image_download;
mod image_extract;
mod image_files;
mod image_registry;
mod image_store;
mod image_verification;
mod images;
mod installation;
mod launches;
mod layout;
mod maintenance;
mod network;
mod network_rules;
mod observations;
mod persistence;
mod process;
mod reporting;
mod runtime;
mod state;
mod system_service;
mod updates;
mod vm_config;
mod vm_files;
mod vms;
mod vsock;
mod vsock_stream;

#[cfg(test)]
#[path = "_tests_/fixtures.rs"]
mod fixtures;

#[cfg(test)]
#[path = "_tests_/job_facts_tests.rs"]
mod job_facts_tests;

#[cfg(test)]
#[path = "_tests_/startup_tests.rs"]
mod startup_tests;

#[cfg(test)]
#[path = "_tests_/network_tests.rs"]
mod network_tests;

#[cfg(test)]
#[path = "_tests_/capacity_tests.rs"]
mod capacity_tests;

#[cfg(test)]
#[path = "_tests_/image_fixtures.rs"]
mod image_fixtures;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    match application::main().await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(error = %error, "agent command failed");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
#[path = "_tests_/admission_tests.rs"]
mod admission_tests;

#[cfg(test)]
#[path = "_tests_/recovery_tests.rs"]
mod recovery_tests;

#[cfg(test)]
#[path = "_tests_/image_response_tests.rs"]
mod image_response_tests;

#[cfg(test)]
#[path = "_tests_/image_concurrency_tests.rs"]
mod image_concurrency_tests;
