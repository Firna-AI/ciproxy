//! One ephemeral GitHub Actions runner inside a fresh ciproxy job image.

pub mod boundary;
pub mod error;
pub mod service;

mod boat;
mod cli;
mod process;
mod vsock;

/// Construct the Linux guest application at the binary composition root.
pub use cli::{Application, Cli, LinuxApplication};
