//! Literal pricing and timestamp helpers for metering integration tests.

use chrono::{DateTime, Utc};
use ciproxy_metering::pricing::{AddOns, Plan, Tier};
use rust_decimal::Decimal;

pub(super) fn time(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}
pub(super) fn usd(value: &str) -> Decimal {
    value.parse().unwrap()
}
pub(super) fn free() -> Plan {
    Plan {
        tier: Tier::Free,
        add_ons: AddOns::default(),
    }
}
