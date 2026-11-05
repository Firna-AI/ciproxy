//! Caller-calculated monthly amounts and atomic reservation outcomes.

use rust_decimal::Decimal;

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
    /// The delivery and its sealed launch input committed together.
    Reserved,
    /// Current settings, key, organization, or label forbid a start.
    NotPermitted,
    /// The current saved limit does not cover all required monthly amounts.
    SpendLimit,
}
