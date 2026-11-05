//! Composition root of the open-source ciproxy machine agent.

mod application;
mod area;
mod area_files;
mod area_mounts;
mod area_types;
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
mod host_composition;
mod host_file_ops;
mod host_packages;
mod host_paths;
mod host_preparation;
mod host_release;
mod host_system;
mod host_tool_files;
mod host_tools;
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
mod logging;
mod maintenance;
mod mode;
mod network;
mod network_rules;
mod observations;
mod persistence;
mod process;
mod reporting;
mod runtime;
mod runtime_reports;
mod runtime_worker;
mod sparse_copy;
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

#[cfg(test)]
#[path = "_tests_/mode_reconcile_tests.rs"]
mod mode_reconcile_tests;

#[cfg(test)]
#[path = "_tests_/timed_fakes.rs"]
mod timed_fakes;

#[cfg(test)]
#[path = "_tests_/runtime_fakes.rs"]
mod runtime_fakes;

#[cfg(test)]
#[path = "_tests_/host_fixtures.rs"]
mod host_fixtures;

#[cfg(test)]
#[path = "_tests_/host_lifecycle_fakes.rs"]
mod host_lifecycle_fakes;

#[cfg(test)]
#[path = "_tests_/area_fixtures.rs"]
mod area_fixtures;
