//! Hosted GitHub App, repository policies, and durable scale-set sessions.

#![warn(unreachable_pub)]

pub mod app;
pub mod assignments;
pub mod auth;
mod delivery_throttle;
pub mod delivery_worker;
pub mod error;
pub mod hooks;
pub mod installation_state;
pub mod listener;
pub mod persistence;
pub mod redelivery;
pub mod registrations;
pub mod rest;
pub mod runtime;
pub mod timing;

pub mod scale_sets;
#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
