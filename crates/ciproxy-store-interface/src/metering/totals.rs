//! Workspace UTC daily totals kept for ninety days on every tier.

use std::collections::BTreeMap;

use chrono::{DateTime, NaiveDate, Utc};
use ciproxy_protocol::resources::VmSize;
use rust_decimal::Decimal;

use crate::metering::facts::{JobPlace, UsageJob};
use crate::metering::history::OverflowCost;
use crate::overflow::provider::OverflowProvider;

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
    /// Jobs on ciproxy-created runners of each overflow provider.
    pub overflow: BTreeMap<OverflowProvider, LocationTotals>,
}

impl SizeTotal {
    /// Start with zero usage for this size.
    pub fn new(size: VmSize) -> Self {
        Self {
            size,
            machines: LocationTotals::default(),
            overflow: BTreeMap::new(),
        }
    }

    /// Get the usage of one place, creating an empty total when needed.
    pub fn usage_mut(&mut self, place: JobPlace) -> &mut LocationTotals {
        match place {
            JobPlace::Machine => &mut self.machines,
            JobPlace::Overflow { provider } => self.overflow.entry(provider).or_default(),
        }
    }
}

/// Daily execution totals for every organization of a workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyTotal {
    /// The UTC date of these totals.
    pub day: NaiveDate,
    /// Totals for each actual VM size used that day.
    pub sizes: Vec<SizeTotal>,
    /// Cost of each provider's sandboxes, including those with no job.
    pub overflow_usd: BTreeMap<OverflowProvider, Decimal>,
}

impl DailyTotal {
    /// Start with zero usage and cost for this day.
    pub fn new(day: NaiveDate) -> Self {
        Self {
            day,
            sizes: Vec::new(),
            overflow_usd: BTreeMap::new(),
        }
    }
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
    /// The place that ran these jobs.
    pub place: JobPlace,
    /// The change to starts at this place.
    pub starts: i64,
    /// The change to elapsed minutes at this place.
    pub minutes: Decimal,
}

/// The overflow facts and observation time used by the previous cost calculation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountedOverflow {
    /// The stored start and runtime rates of a ciproxy sandbox.
    pub cost: OverflowCost,
    /// The explicit time through which runtime cost was accounted.
    pub observed_at: DateTime<Utc>,
}

/// An anonymous USD correction for one provider and UTC date.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyOverflowCost {
    /// The UTC date to update.
    pub day: NaiveDate,
    /// The provider that bills the sandbox.
    pub provider: OverflowProvider,
    /// The signed change in ciproxy overflow costs.
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
