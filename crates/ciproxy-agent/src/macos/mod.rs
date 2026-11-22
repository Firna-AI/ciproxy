//! macOS host parts with no macOS-only dependency, so Linux tests run them.
//!
//! The [macOS host design](../../macos-host.md) gives the whole design.
//! `platform::macos` composes these parts with `ciproxy-vz`. Until it does,
//! they have no caller outside their tests.
#![allow(dead_code)]

mod fd_passing;
mod vm_control;
mod vm_control_client;
mod vm_control_server;

#[cfg(test)]
#[path = "_tests_/vm_control_tests.rs"]
mod vm_control_tests;

#[cfg(test)]
#[path = "_tests_/vm_control_socket_tests.rs"]
mod vm_control_socket_tests;

mod launchd;
mod paths;

#[cfg(test)]
#[path = "_tests_/launchd_tests.rs"]
mod launchd_tests;
