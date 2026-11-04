//! Literal fixtures shared by metering boundary tests.

use chrono::{DateTime, NaiveDate, Utc};
use ciproxy_store_interface::metering::facts::{JobPlace, UsageJob};
use ciproxy_store_interface::metering::history::{CostHistory, PlanCost};
use ciproxy_store_interface::vm_size::VmSize;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::pricing::{AddOns, Plan, Tier};
use crate::report::catalog::{Alternative, PriceList, SizePrice};
use crate::report::range::DateRange;

pub(crate) fn at(value: &str) -> DateTime<Utc> {
    value.parse().unwrap()
}
pub(crate) fn day(value: &str) -> NaiveDate {
    value.parse().unwrap()
}
pub(crate) fn usd(value: &str) -> Decimal {
    value.parse().unwrap()
}
pub(crate) fn size() -> VmSize {
    VmSize {
        vcpus: 2,
        memory_gb: 8,
        disk_gb: 60,
    }
}
pub(crate) fn plan(tier: Tier) -> Plan {
    Plan {
        tier,
        add_ons: AddOns::default(),
    }
}
pub(crate) fn range(from: &str, until: &str) -> DateRange {
    DateRange {
        from: day(from),
        until: day(until),
    }
}

pub(crate) fn job(id: u128, start: &str, end: Option<&str>, place: JobPlace) -> UsageJob {
    UsageJob {
        id: Uuid::from_u128(id),
        organization_id: Uuid::from_u128(100),
        started_at: at(start),
        completed_at: end.map(at),
        size: size(),
        place,
    }
}

pub(crate) fn catalog(alternative: Alternative, minute_usd: &str) -> PriceList {
    PriceList {
        alternative,
        dated: day("2026-11-04"),
        sizes: vec![SizePrice {
            vcpus: 2,
            memory_gb: 8,
            minute_usd: usd(minute_usd),
        }],
    }
}

pub(crate) fn free_history() -> CostHistory {
    CostHistory {
        workspace_started_at: Some(at("2026-01-01T00:00:00Z")),
        plans: vec![PlanCost {
            id: Uuid::from_u128(1),
            starts_at: at("2026-01-01T00:00:00Z"),
            ends_at: None,
            monthly_usd: Decimal::ZERO,
        }],
        ..CostHistory::default()
    }
}
