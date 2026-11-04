//! Real HTTP and filesystem integration fixtures.

mod fixture;
mod github;
pub mod job;
pub mod relay;
pub mod uploads;

pub use fixture::{Fixture, facts};

pub const V2: &str = "/twirp/github.actions.results.api.v1.CacheService/";
