//! Streaming disk storage, immutable entries, and local eviction.

pub mod capture;
pub mod disk;
pub mod interface;

mod allocation;
mod assembly;
mod index;
mod maintenance;
mod publication;
mod read;
