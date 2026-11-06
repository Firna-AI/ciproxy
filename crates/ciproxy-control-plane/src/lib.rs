//! Composition and lifecycle of the hosted ciproxy HTTP service.

#![warn(unreachable_pub)]

pub mod accounting;
mod actions;
pub mod application;
mod capacity;
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
mod reconciliation;
mod repositories;
mod retirement;
pub mod runtime;
mod slots;
mod snapshot;
mod startup_api;
mod tasks;
mod warm;
