//! Workspace UTC daily totals kept for ninety days on every tier.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::metering::facts::UsageJob;
use crate::metering::history::BoatCost;
use ciproxy_protocol::resources::VmSize;

/// The starts and elapsed runtime of one execution location.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocationTotals {
    /// The jobs whose runners took them in this day.
    pub starts: u64,
    /// The elapsed job minutes that overlap this day.
    pub minutes: Decimal,
}

/// Daily execution totals for one actual VM size, without job identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SizeTotal {
    /// The actual VM resources used for alternative-price matching.
    pub size: VmSize,
    /// Jobs on the workspace's own machines.
    pub machines: LocationTotals,
    /// Jobs on ciproxy-created Boat runners.
    pub boat: LocationTotals,
}

/// Daily execution totals for every organization of a workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyTotal {
    /// The UTC date of these totals.
    pub day: NaiveDate,
    /// Totals for each actual VM size used that day.
    pub sizes: Vec<SizeTotal>,
    /// Cost of all ciproxy Boat sandboxes, including those with no job.
    pub boat_usd: Decimal,
}

/// The previous job facts and observation time used for incremental accounting.
///
/// This cursor is deleted with its job. Anonymous daily totals remain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountedJob {
    /// The facts used by the previous calculation.
    pub job: UsageJob,
    /// The explicit time through which running runtime was accounted.
    pub observed_at: DateTime<Utc>,
}

/// A signed correction to one anonymous daily aggregate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyDelta {
    /// The UTC date to update.
    pub day: NaiveDate,
    /// The actual VM resources of this aggregate.
    pub size: VmSize,
    /// The change to own-machine starts.
    pub machine_starts: i64,
    /// The change to own-machine elapsed minutes.
    pub machine_minutes: Decimal,
    /// The change to Boat starts.
    pub boat_starts: i64,
    /// The change to Boat elapsed minutes.
    pub boat_minutes: Decimal,
}

/// The Boat facts and observation time used by the previous cost calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountedBoat {
    /// The stored start and runtime rates of a ciproxy sandbox.
    pub cost: BoatCost,
    /// The explicit time through which runtime cost was accounted.
    pub observed_at: DateTime<Utc>,
}

/// An anonymous USD correction for one UTC date.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyBoatCost {
    /// The UTC date to update.
    pub day: NaiveDate,
    /// The signed change in ciproxy Boat costs.
    pub usd: Decimal,
}

/// Anonymous start counts with exact UTC timestamps for billing boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartTotal {
    /// The runner-taken timestamp, with no job or organization identity.
    pub taken_at: DateTime<Utc>,
    /// The number of starts at this instant, signed for corrections.
    pub starts: i64,
}
