//! One ephemeral GitHub Actions runner inside a fresh ciproxy job image.

mod assignment;
pub mod boundary;
pub mod error;
mod guest_setup;
mod handoff;
mod replay;
mod runner_assignment;
pub mod service;

mod cli;
mod platform;
mod power;
mod preparation;
mod process;
mod proxy;
mod sandbox;
mod vsock;

/// Construct the guest application at the binary composition root.
pub use cli::{Application, Cli, GuestApplication};
