//! Composition root of the open-source ciproxy machine agent.

mod application;
mod application_images;
mod area;
mod area_files;
mod area_mounts;
mod area_types;
mod assignment_state;
mod boundaries;
mod budget_file;
mod cache;
mod cache_admin;
mod cache_authority;
mod cache_install;
mod cache_network;
mod cache_service;
mod cache_unit;
mod capacity;
mod cli;
mod commands;
mod connection;
mod connection_socket;
mod connection_types;
mod controller;
mod docker;
mod docker_files;
mod docker_prepare;
mod docker_recovery;
mod error;
mod gateway;
mod guest_transport;
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
mod image_space;
mod image_store;
mod image_verification;
mod images;
mod installation;
mod job_decisions;
mod job_preparation;
mod launches;
mod layout;
mod logging;
mod maintenance;
mod mode;
mod network;
mod network_inventory;
mod network_lifecycle;

#[cfg(test)]
#[path = "_tests_/network_fixtures.rs"]
mod network_fixtures;

#[cfg(test)]
#[path = "_tests_/network_ownership_tests.rs"]
mod network_ownership_tests;

#[cfg(test)]
#[path = "_tests_/network_lifecycle_tests.rs"]
mod network_lifecycle_tests;

#[cfg(test)]
#[path = "_tests_/network_barriers.rs"]
mod network_barriers;

#[cfg(test)]
#[path = "_tests_/network_progress_tests.rs"]
mod network_progress_tests;

#[cfg(test)]
#[path = "_tests_/network_uninstall_tests.rs"]
mod network_uninstall_tests;

mod network_rules;
#[cfg(test)]
#[path = "_tests_/network_start_failure_tests.rs"]
mod network_start_failure_tests;
mod observations;
mod persistence;
mod process;
mod release_client;
mod reporting;
mod runtime;
mod runtime_reports;
mod runtime_worker;
mod slot_allocation;
mod sparse_copy;
mod startup;
mod state;
mod storage_sizing;
mod system_service;
mod terminal_completion;
mod updates;
mod vm_config;
mod vm_files;
mod vm_root;
mod vms;
mod vsock;
mod vsock_stream;

#[cfg(test)]
#[path = "_tests_/vm_root_tests.rs"]
mod vm_root_tests;

#[cfg(test)]
#[path = "_tests_/vm_root_failure_tests.rs"]
mod vm_root_failure_tests;

#[cfg(test)]
#[path = "_tests_/image_start_tests.rs"]
mod image_start_tests;

#[cfg(test)]
#[path = "_tests_/image_retry_tests.rs"]
mod image_retry_tests;

#[cfg(test)]
#[path = "_tests_/fixtures.rs"]
mod fixtures;

#[cfg(test)]
#[path = "_tests_/connection_fixtures.rs"]
mod connection_fixtures;

#[cfg(test)]
#[path = "_tests_/job_facts_tests.rs"]
mod job_facts_tests;

#[cfg(test)]
#[path = "_tests_/cache_authority_tests.rs"]
mod cache_authority_tests;

#[cfg(test)]
#[path = "_tests_/host_decision_tests.rs"]
mod host_decision_tests;

#[cfg(test)]
#[path = "_tests_/hosted_preparation_tests.rs"]
mod hosted_preparation_tests;

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

#[cfg(test)]
#[path = "_tests_/root_failure_fixtures.rs"]
mod root_failure_fixtures;
