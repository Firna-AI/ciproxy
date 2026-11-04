//! Hosted machine registration, commands, and agent WebSocket transport.

#![warn(unreachable_pub)]

pub mod boundary;
pub mod connections;
pub mod error;
pub mod gateway;
pub mod transport;
pub mod types;

mod authentication;
mod management;
mod versions;

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
