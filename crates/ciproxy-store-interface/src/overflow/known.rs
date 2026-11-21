//! Known sandbox identity with its inseparable provider lifetime proof.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// A provider sandbox whose identity and latest possible end were observed together.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KnownSandbox {
    id: String,
    lifetime_ends_by: DateTime<Utc>,
}

impl KnownSandbox {
    /// Establish the original lifetime bound from a successful existence observation.
    pub fn observed(id: String, at: DateTime<Utc>, lifetime_seconds: u32) -> Self {
        Self {
            id,
            lifetime_ends_by: bound(at, lifetime_seconds),
        }
    }

    /// Borrow the proven provider identity.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Latest possible end under the unchanged provider lifetime.
    pub fn lifetime_ends_by(&self) -> DateTime<Utc> {
        self.lifetime_ends_by
    }

    /// Tighten the existing proof with a trusted earlier creation observation.
    pub fn observe_creation(&mut self, at: DateTime<Utc>, lifetime_seconds: u32) {
        self.lifetime_ends_by = self.lifetime_ends_by.min(bound(at, lifetime_seconds));
    }
}

fn bound(at: DateTime<Utc>, lifetime_seconds: u32) -> DateTime<Utc> {
    at.checked_add_signed(Duration::seconds(i64::from(lifetime_seconds)))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}
