//! One ephemeral GitHub Actions runner inside a fresh ciproxy job image.

mod assignment;
pub mod boundary;
pub mod error;
mod handoff;
pub mod service;

mod boat;
mod cli;
mod preparation;
mod process;
mod proxy;
mod vsock;

/// Construct the Linux guest application at the binary composition root.
pub use cli::{Application, Cli, LinuxApplication};
