//! Secret values with explicit access and a redacted diagnostic form.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A credential, token, registration, or URL that must not enter `Debug`.
///
/// JSON serialization carries the original value for the intended recipient.
/// This type does not encrypt or erase its allocation.
#[derive(Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Secret(String);

impl Secret {
    /// Wrap a value whose diagnostic output must be redacted.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Expose the value explicitly for authentication or a download.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret([REDACTED])")
    }
}

#[cfg(test)]
#[path = "_tests_/secret_tests.rs"]
mod secret_tests;
