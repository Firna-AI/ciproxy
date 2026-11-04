//! Streaming disk storage, immutable entries, and local eviction.

pub mod capture;
pub mod disk;
pub mod interface;

mod assembly;
mod index;
mod publication;
mod read;
