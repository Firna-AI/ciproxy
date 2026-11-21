//! Source-hook integration through private production socket implementations.

#![expect(
    dead_code,
    reason = "The integration harness imports the socket boundary and its production dependencies."
)]

#[path = "../src/assignment.rs"]
mod assignment;
#[path = "../src/error.rs"]
mod error;
#[path = "../src/guest_setup.rs"]
mod guest_setup;
#[path = "../src/handoff.rs"]
mod handoff;
#[path = "../src/proxy.rs"]
mod proxy;

#[path = "_tests_/assignment_socket_tests.rs"]
mod assignment_socket_tests;
