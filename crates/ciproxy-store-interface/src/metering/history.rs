//! Exact USD price and billing-period snapshots supplied by trusted services.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Half-open boundaries of one Free or Stripe billing period.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Period {
    /// The first instant counted in this period.
    pub starts_at: DateTime<Utc>,
    /// The first instant of the next period.
    pub ends_at: DateTime<Utc>,
}

/// A customer's monthly own-machine cost during one price interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineCost {
    /// The stable price-history identifier for idempotent writes.
    pub id: Uuid,
    /// The machine to which this cost belongs.
    pub machine_id: Uuid,
    /// The first instant with this monthly cost.
    pub starts_at: DateTime<Utc>,
    /// The end of this price interval, absent for the current cost.
    pub ends_at: Option<DateTime<Utc>>,
    /// The customer's monthly cost in USD, without display rounding.
    pub monthly_usd: Decimal,
}

/// The monthly tier and add-on total during one price interval.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlanCost {
    /// The stable price-history identifier for idempotent writes.
    pub id: Uuid,
    /// When the confirmed tier or add-on total took effect.
    pub starts_at: DateTime<Utc>,
    /// When this price ended, absent for the current price.
    pub ends_at: Option<DateTime<Utc>>,
    /// The monthly tier and add-on total in USD.
    pub monthly_usd: Decimal,
}

/// Start and runtime prices for one ciproxy-created Boat sandbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatCost {
    /// The runner whose sandbox ciproxy created.
    pub runner_id: Uuid,
    /// When Boat started billing this sandbox.
    pub starts_at: DateTime<Utc>,
    /// When billing ended, absent while the sandbox runs.
    pub ends_at: Option<DateTime<Utc>>,
    /// The sandbox's one-time start charge in USD.
    pub start_usd: Decimal,
    /// The sandbox's runtime cost per elapsed minute in USD.
    pub minute_usd: Decimal,
}

/// Stored histories used by savings calculations.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CostHistory {
    /// The trusted workspace creation time, absent for legacy records.
    pub workspace_started_at: Option<DateTime<Utc>>,
    /// Monthly own-machine cost intervals.
    pub machines: Vec<MachineCost>,
    /// Confirmed monthly tier and add-on price intervals.
    pub plans: Vec<PlanCost>,
    /// Ciproxy-created Boat sandboxes, including those that took no job.
    pub boat: Vec<BoatCost>,
}
