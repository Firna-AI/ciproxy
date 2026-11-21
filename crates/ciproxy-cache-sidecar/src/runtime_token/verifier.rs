//! RS256 verification before any token claim grants machine-cache access.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::jobs::{JobFacts, JobScope, UnknownJobScopeReason};
use ciproxy_protocol::secret::Secret;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, Validation, decode, decode_header};

use crate::clock::Clock;

use super::RuntimeTokens;
use super::claims::Claims;
use super::keys::SigningKeys;

/// Machine-side verifier with an injected clock and public signing keys.
pub struct TokenVerifier {
    keys: Arc<dyn SigningKeys>,
    clock: Arc<dyn Clock>,
}

impl TokenVerifier {
    /// Compose at the machine boundary, never from job-controlled configuration.
    pub fn new(keys: Arc<dyn SigningKeys>, clock: Arc<dyn Clock>) -> Self {
        Self { keys, clock }
    }

    async fn checked(
        &self,
        token: &Secret,
        facts: &JobFacts,
    ) -> Result<JobScope, UnknownJobScopeReason> {
        let token = token.expose_secret();
        if token.is_empty() {
            return Err(UnknownJobScopeReason::MissingToken);
        }
        let Ok(header) = decode_header(token) else {
            return Err(UnknownJobScopeReason::MalformedToken);
        };
        if header.alg != Algorithm::RS256 {
            return Err(UnknownJobScopeReason::UnsupportedAlgorithm);
        }
        let Some(kid) = header.kid.filter(|value| !value.is_empty()) else {
            return Err(UnknownJobScopeReason::MissingKeyId);
        };
        let key = self.keys.key(&kid).await?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.required_spec_claims.clear();
        validation.validate_exp = false;
        validation.validate_nbf = false;
        validation.validate_aud = false;
        let claims = match decode::<Claims>(token, &key, &validation) {
            Ok(data) => data.claims,
            Err(error) => {
                return Err(match error.kind() {
                    ErrorKind::Json(_)
                    | ErrorKind::InvalidClaimFormat(_)
                    | ErrorKind::MissingRequiredClaim(_) => UnknownJobScopeReason::MalformedToken,
                    _ => UnknownJobScopeReason::InvalidSignature,
                });
            }
        };
        claims.scope(facts, self.clock.now())
    }
}

#[async_trait]
impl RuntimeTokens for TokenVerifier {
    async fn check(&self, token: &Secret, facts: &JobFacts) -> JobScope {
        match self.checked(token, facts).await {
            Ok(scope) => scope,
            Err(reason) => JobScope::Unknown { reason },
        }
    }
}
