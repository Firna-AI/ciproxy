//! Hosted execution and cleanup for ciproxy's Boat overflow runners.

#![warn(unreachable_pub)]

pub mod api;
pub mod boat;
mod budget;
mod catalog;
pub mod clock;
mod cost_state;
pub mod crypto;
pub mod error;
pub mod kms;
pub mod provider;
pub mod secrets;
pub mod service;
mod session;
mod spend;
pub mod view;

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
