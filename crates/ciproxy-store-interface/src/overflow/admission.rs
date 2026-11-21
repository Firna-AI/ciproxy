//! Caller-calculated monthly amounts and atomic reservation outcomes.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::labels::Label;
use crate::overflow::record::{ProviderParameters, StartPhase, StartRecord};

/// Current persisted facts used by the service's pure resource selection rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DispatchFacts {
    /// Current label dimensions, destinations, and nothing-fits choice.
    pub label: Label,
}

/// One selected provider plan and its complete budget, accepted as one store operation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreparedDispatch {
    /// Exact provider request and price selected from current facts.
    pub parameters: ProviderParameters,
    /// Actual dispatch clock boundary used for lifetime and UTC-month amounts.
    pub starts_at: DateTime<Utc>,
    /// Complete required amounts, including this exact plan.
    pub budget: ReservationBudget,
}

impl PreparedDispatch {
    /// Apply only the fields that become fixed when pending demand is dispatched.
    pub fn apply(&self, pending: &StartRecord) -> StartRecord {
        let mut record = pending.clone();
        record.dispatch = Some(self.parameters.clone());
        record.starts_at = self.starts_at;
        record.phase = StartPhase::Creating;
        record
    }
}

/// Exact counted amounts including the proposed delivery in each affected month.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReservationBudget {
    /// Required USD in the delivery's UTC start month.
    pub current_month_usd: Decimal,
    /// Required USD in the next UTC month, when the lifetime crosses its boundary.
    pub next_month_usd: Option<Decimal>,
}

/// The result after comparing current settings and tenant authority atomically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReservationOutcome {
    /// Current authority and budget passed, and the fixed dispatch intent committed.
    Reserved,
    /// Current settings, key, organization, or label forbid a start.
    NotPermitted,
    /// The current saved limit does not cover all required monthly amounts.
    SpendLimit,
}
