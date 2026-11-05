//! Real HTTP and filesystem integration fixtures.

mod fixture;
mod github;
pub mod job;
mod registration;
pub mod relay;
pub mod tokens;
pub mod uploads;

pub use fixture::Fixture;
pub use tokens::facts;

pub const V2: &str = "/twirp/github.actions.results.api.v1.CacheService/";
