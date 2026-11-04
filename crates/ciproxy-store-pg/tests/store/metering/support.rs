//! Literal pricing and timestamp helpers for metering integration tests.

use chrono::{DateTime, Utc};
use ciproxy_metering::pricing::{AddOns, Plan, Tier};
use ciproxy_metering::report::catalog::{Alternative, PriceList, SizePrice};
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

pub(super) fn prices() -> [PriceList; 2] {
    [
        PriceList {
            alternative: Alternative::Blacksmith,
            dated: time("2026-11-04T00:00:00Z").date_naive(),
            sizes: vec![SizePrice {
                vcpus: 2,
                memory_gb: 8,
                minute_usd: usd("1"),
            }],
        },
        PriceList {
            alternative: Alternative::Github,
            dated: time("2026-11-04T00:00:00Z").date_naive(),
            sizes: vec![SizePrice {
                vcpus: 2,
                memory_gb: 8,
                minute_usd: usd("2"),
            }],
        },
    ]
}
