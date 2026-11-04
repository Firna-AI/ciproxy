//! Opaque values that never print secret material.

use std::fmt;

/// A token digest computed by the caller before persistence.
#[derive(Clone, Eq, PartialEq)]
pub struct TokenHash(Vec<u8>);

impl TokenHash {
    /// Wrap an already computed digest.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrow the digest for persistence or verification.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for TokenHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TokenHash([redacted])")
    }
}

/// A Boat key encrypted by the caller.
#[derive(Clone, Eq, PartialEq)]
pub struct EncryptedBoatKey(Vec<u8>);

impl EncryptedBoatKey {
    /// Wrap ciphertext, including the caller's encryption envelope.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrow the ciphertext for persistence or decryption.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for EncryptedBoatKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EncryptedBoatKey([redacted])")
    }
}

#[cfg(test)]
#[path = "_tests_/secrets_tests.rs"]
mod secrets_tests;
