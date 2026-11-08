//! Composition and lifecycle of the hosted ciproxy HTTP service.

#![warn(unreachable_pub)]

pub mod accounting;
mod actions;
pub mod application;
pub mod billing;
pub mod boat_authority;
pub mod boat_callback;
mod boat_jobs;
mod boat_retirement;
mod boat_runtime;
mod boat_snapshot;
mod capacity;
mod comparison_prices;
pub mod composition;
pub mod config;
mod docker_budget;
mod docker_delivery;
pub mod error;
mod events;
pub mod http;
mod image_delivery;
pub mod image_release;
pub mod image_signing;
mod lifecycle;
mod listeners;
mod overflow_composition;
pub mod pages;
mod reconciliation;
mod repositories;
mod reservations;
mod retirement;
pub mod runtime;
mod slots;
mod snapshot;
mod startup_api;
mod startup_billing;
mod takeout;
mod tasks;
mod warm;
