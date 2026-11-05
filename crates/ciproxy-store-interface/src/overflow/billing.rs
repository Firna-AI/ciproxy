//! Finite established billing coverage bounded by its provider observation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A measured billing prefix that cannot publish an open or future interval.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MeasuredBillingInterval {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

impl MeasuredBillingInterval {
    /// Clamp derived coverage to the latest observation and return whether it changed.
    pub fn new(
        starts_at: DateTime<Utc>,
        derived_end: DateTime<Utc>,
        observed_until: DateTime<Utc>,
    ) -> (Self, bool) {
        let bounded_start = starts_at.min(observed_until);
        let ends_at = derived_end.max(bounded_start).min(observed_until);
        (
            Self {
                starts_at: bounded_start,
                ends_at,
            },
            starts_at != bounded_start || derived_end != ends_at,
        )
    }

    /// First instant represented by this finite price interval.
    pub fn starts_at(self) -> DateTime<Utc> {
        self.starts_at
    }

    /// Exclusive end, always present and no later than the observation.
    pub fn ends_at(self) -> DateTime<Utc> {
        self.ends_at
    }
}
