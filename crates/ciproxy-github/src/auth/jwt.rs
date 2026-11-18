//! In-memory RSA signing with bounded GitHub App JWT lifetimes.

use std::sync::Arc;

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use secrecy::SecretString;
use serde::Serialize;

use crate::error::{Result, ResultExt};
use crate::runtime::Runtime;

use super::AppJwt;

/// A control-plane RSA signer. Its key never leaves this object.
pub struct RsaAppJwt {
    issuer: String,
    key: EncodingKey,
    runtime: Arc<dyn Runtime>,
}

#[derive(Serialize)]
struct Claims<'a> {
    iss: &'a str,
    iat: i64,
    exp: i64,
}

impl RsaAppJwt {
    /// Parse the App's RSA key at the composition root.
    pub fn from_pem(issuer: String, pem: &[u8], runtime: Arc<dyn Runtime>) -> Result<Self> {
        Ok(Self {
            issuer,
            key: EncodingKey::from_rsa_pem(pem).internal()?,
            runtime,
        })
    }
}

impl AppJwt for RsaAppJwt {
    fn token(&self) -> Result<SecretString> {
        let now = self.runtime.now().timestamp();
        let claims = Claims {
            iss: &self.issuer,
            iat: now - 60,
            exp: now + 480,
        };
        Ok(encode(&Header::new(Algorithm::RS256), &claims, &self.key)
            .internal()?
            .into())
    }
}

#[cfg(test)]
#[path = "_tests_/jwt_tests.rs"]
mod jwt_tests;
