//! Stable identifiers for supported overflow providers.

use serde::{Deserialize, Serialize};

/// The provider that owns an overflow account, sandbox, or cost.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OverflowProvider {
    /// Boat, also the provider of records saved before provider identity.
    #[default]
    Boat,
}

impl OverflowProvider {
    /// The stored text identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Boat => "boat",
        }
    }
}
