//! Composition root of the open-source ciproxy machine agent.

#![cfg_attr(
    not(target_os = "linux"),
    expect(
        dead_code,
        reason = "Only the Linux adapters use some shared parts until macOS has its adapters."
    )
)]

mod application;
#[cfg(target_os = "linux")]
mod application_cache;
mod application_endpoint;
#[cfg(target_os = "linux")]
mod application_images;
mod application_metadata;
mod application_setup;
mod application_uninstall;
#[cfg(target_os = "linux")]
mod area;
#[cfg(target_os = "linux")]
mod area_files;
#[cfg(target_os = "linux")]
mod area_mounts;
#[cfg(target_os = "linux")]
mod area_types;
mod assignment_state;
mod boundaries;
#[cfg(target_os = "linux")]
mod budget_file;
#[cfg(target_os = "linux")]
mod cache;
#[cfg(target_os = "linux")]
mod cache_admin;
mod cache_authority;
#[cfg(target_os = "linux")]
mod cache_composition;
#[cfg(target_os = "linux")]
mod cache_install;
#[cfg(target_os = "linux")]
mod cache_network;
#[cfg(target_os = "linux")]
mod cache_service;
#[cfg(target_os = "linux")]
mod cache_unit;
mod capacity;
mod cli;
mod commands;
mod connection;
mod connection_socket;
mod connection_types;
mod controller;
#[cfg(target_os = "linux")]
mod docker;
#[cfg(target_os = "linux")]
mod docker_files;
#[cfg(target_os = "linux")]
mod docker_prepare;
#[cfg(target_os = "linux")]
mod docker_recovery;
#[cfg(target_os = "linux")]
mod docker_reservation;
mod endpoint;
mod endpoint_files;
mod error;
mod gateway;
mod guest_transport;
#[cfg(target_os = "linux")]
mod hardware;
#[cfg(target_os = "linux")]
mod host_composition;
#[cfg(target_os = "linux")]
mod host_file_ops;
#[cfg(target_os = "linux")]
mod host_packages;
mod host_paths;
#[cfg(target_os = "linux")]
mod host_preparation;
#[cfg(target_os = "linux")]
mod host_release;
#[cfg(target_os = "linux")]
mod host_system;
#[cfg(target_os = "linux")]
mod host_tool_files;
#[cfg(target_os = "linux")]
mod host_tools;
mod identity_store;
#[cfg(target_os = "linux")]
mod image_artifact;
#[cfg(target_os = "linux")]
mod image_download;
#[cfg(target_os = "linux")]
mod image_extract;
#[cfg(target_os = "linux")]
mod image_files;
#[cfg(target_os = "linux")]
mod image_registry;
#[cfg(target_os = "linux")]
mod image_space;
#[cfg(target_os = "linux")]
mod image_store;
#[cfg(target_os = "linux")]
mod image_verification;
#[cfg(target_os = "linux")]
mod images;
mod installation;
mod job_decisions;
mod job_preparation;
mod launches;
#[cfg(target_os = "linux")]
mod layout;
mod logging;
#[cfg(any(target_os = "macos", test))]
mod macos;
mod maintenance;
mod mode;
#[cfg(target_os = "linux")]
mod network;
#[cfg(target_os = "linux")]
mod network_forwarding;
#[cfg(target_os = "linux")]
mod network_inventory;
mod network_lifecycle;
#[cfg(target_os = "linux")]
mod network_preparation;
#[cfg(target_os = "linux")]
mod network_rules;
mod observations;
mod persistence;
mod platform;
mod preparation;
#[cfg(target_os = "linux")]
mod process;
mod registration;
mod release_client;
mod reporting;
mod retirement;
mod runtime;
mod runtime_ownership;
mod runtime_reports;
mod runtime_worker;
mod slot_allocation;
mod sparse_copy;
mod startup;
mod state;
#[cfg(target_os = "linux")]
mod storage_sizing;
#[cfg(target_os = "linux")]
mod system_service;
mod terminal_completion;
mod unit_state;
#[cfg(target_os = "linux")]
mod updates;
mod updates_checked;
#[cfg(target_os = "linux")]
mod vm_config;
#[cfg(target_os = "linux")]
mod vm_drives;
#[cfg(target_os = "linux")]
mod vm_files;
mod vm_liveness;
#[cfg(target_os = "linux")]
mod vm_root;
#[cfg(target_os = "linux")]
mod vms;
mod vsock;
mod vsock_stream;
mod workload_space;

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
mod root_tests;

#[cfg(test)]
#[path = "_tests_/network_fixtures.rs"]
mod network_fixtures;

#[cfg(test)]
#[path = "_tests_/network_barriers.rs"]
mod network_barriers;

#[cfg(test)]
#[path = "_tests_/fixtures.rs"]
mod fixtures;

#[cfg(test)]
#[path = "_tests_/connection_fixtures.rs"]
mod connection_fixtures;

#[cfg(test)]
#[path = "_tests_/cache_authority_tests.rs"]
mod cache_authority_tests;

#[cfg(test)]
#[path = "_tests_/image_fixtures.rs"]
mod image_fixtures;

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
#[path = "_tests_/identity_store_fixtures.rs"]
mod identity_store_fixtures;

#[cfg(test)]
#[path = "_tests_/host_lifecycle_fakes.rs"]
mod host_lifecycle_fakes;

#[cfg(test)]
#[path = "_tests_/area_fixtures.rs"]
mod area_fixtures;

#[cfg(test)]
#[path = "_tests_/root_failure_fixtures.rs"]
mod root_failure_fixtures;
