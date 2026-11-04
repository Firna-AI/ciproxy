//! Protocol version used by both ciproxy binaries.

use serde::{Deserialize, Serialize};

/// A protocol version encoded as a JSON integer.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ProtocolVersion {
    number: u16,
}

impl ProtocolVersion {
    /// The protocol version used by this workspace.
    pub const CURRENT: Self = Self { number: 1 };
}

#[cfg(test)]
#[path = "_tests_/version_tests.rs"]
mod version_tests;
