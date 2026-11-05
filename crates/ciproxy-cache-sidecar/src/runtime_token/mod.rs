//! GitHub-signed runtime scopes checked once at the machine boundary.

pub mod keys;
pub mod source;

mod claims;
mod verifier;

use async_trait::async_trait;
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::secret::Secret;

pub use verifier::TokenVerifier;

/// Verification boundary consumed by the authenticated machine service.
#[async_trait]
pub trait RuntimeTokens: Send + Sync {
    /// Return checked access scopes or a safe unknown reason for this assignment.
    async fn check(&self, token: &Secret, facts: &JobFacts) -> JobScope;
}

#[cfg(test)]
#[path = "_tests_/mod.rs"]
mod tests;
