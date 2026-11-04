//! Hosted GitHub App, repository policies, and durable scale-set sessions.

#![warn(unreachable_pub)]

pub mod app;
pub mod auth;
pub mod error;
pub mod hooks;
pub mod listener;
pub mod persistence;
pub mod registrations;
pub mod rest;
pub mod runtime;

pub mod scale_sets;
#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
