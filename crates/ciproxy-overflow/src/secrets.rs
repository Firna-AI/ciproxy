//! Secret bytes, random callback credentials, and digest-only lookup.

use std::fmt;

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::secrets::TokenHash;
use rand::RngExt;
use sha2::{Digest, Sha256};
use unimock::unimock;
use zeroize::Zeroizing;

/// Secret serialized input that erases its byte allocation on release.
#[derive(Clone, Eq, PartialEq)]
pub struct SecretBytes(Zeroizing<Vec<u8>>);

impl SecretBytes {
    /// Wrap bytes that must not appear in diagnostics.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(Zeroizing::new(bytes))
    }
    /// Borrow bytes only for their intended encrypted or authenticated boundary.
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SecretBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretBytes([redacted])")
    }
}

/// The entropy boundary for sandbox-only callback tokens.
#[unimock(api = TokenSourceMock)]
pub trait TokenSource: Send + Sync {
    /// Generate a fresh 256-bit token encoded without shell metacharacters.
    fn token(&self) -> Secret;
}

/// Operating-system seeded callback token generation.
pub struct RandomTokens;

impl TokenSource for RandomTokens {
    fn token(&self) -> Secret {
        let bytes: [u8; 32] = rand::rng().random();
        Secret::new(URL_SAFE_NO_PAD.encode(bytes))
    }
}

pub(crate) fn digest(token: &Secret) -> TokenHash {
    TokenHash::new(Sha256::digest(token.expose_secret().as_bytes()).to_vec())
}
