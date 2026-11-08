//! Stripe billing orchestration for hosted ciproxy workspaces.

#![warn(unreachable_pub)]

pub mod access;
pub mod alerts;
pub mod catalog;
pub mod clock;
pub mod error;
pub mod service;
pub mod stripe;
pub mod webhook;

#[cfg(test)]
#[path = "_tests_/access_tests.rs"]
mod access_tests;
#[cfg(test)]
#[path = "_tests_/signature_tests.rs"]
mod signature_tests;
#[cfg(test)]
#[path = "_tests_/webhook_tests.rs"]
mod webhook_tests;
