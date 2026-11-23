//! macOS host parts with no macOS-only dependency, so Linux tests run them.
//!
//! The [macOS host design](../../macos-host.md) gives the whole design.
//! `platform::macos` composes these parts with `ciproxy-vz`. Other targets
//! build this module only for its tests.
#![cfg_attr(
    not(target_os = "macos"),
    allow(dead_code, reason = "Only the macOS platform calls these parts.")
)]

pub(crate) mod fd_passing;
pub(crate) mod guests;
pub(crate) mod vm_control;
pub(crate) mod vm_control_client;
pub(crate) mod vm_control_server;

#[cfg(test)]
#[path = "_tests_/vm_control_tests.rs"]
mod vm_control_tests;

#[cfg(test)]
#[path = "_tests_/vm_control_socket_tests.rs"]
mod vm_control_socket_tests;

pub(crate) mod launchd;
pub(crate) mod log_file;
pub(crate) mod paths;

#[cfg(test)]
#[path = "_tests_/launchd_tests.rs"]
mod launchd_tests;

pub(crate) mod pf;

#[cfg(test)]
#[path = "_tests_/pf_tests.rs"]
mod pf_tests;

pub(crate) mod seats;
pub(crate) mod vm_spec;

#[cfg(test)]
#[path = "_tests_/seats_tests.rs"]
mod seats_tests;

pub(crate) mod hardware;

#[cfg(test)]
#[path = "_tests_/hardware_tests.rs"]
mod hardware_tests;

#[cfg(test)]
#[path = "_tests_/guests_tests.rs"]
mod guests_tests;

pub(crate) mod image_bundle;
pub(crate) mod image_extract;

#[cfg(test)]
#[path = "_tests_/image_bundle_tests.rs"]
mod image_bundle_tests;

pub(crate) mod area_disk;

#[cfg(test)]
#[path = "_tests_/area_disk_tests.rs"]
mod area_disk_tests;

pub(crate) mod image_import;

#[cfg(test)]
#[path = "_tests_/image_import_tests.rs"]
mod image_import_tests;
