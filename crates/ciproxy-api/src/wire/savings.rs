//! Exact-runtime savings reports with presentation-rounded integer cents.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use super::jobs::JobVmSize;
use super::money::Cents;
use super::pages::PageBanner;

/// One of the three UTC report ranges.
#[derive(Clone, Copy, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SavingsRange {
    /// Thirty UTC dates, including today.
    #[default]
    Last30Days,
    /// Current calendar month through today.
    ThisMonth,
    /// Complete previous calendar month.
    LastMonth,
}

/// Savings page selection, defaulting to the last thirty UTC dates.
#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct SavingsQuery {
    /// UTC range to show.
    #[serde(default)]
    pub range: SavingsRange,
}

/// One comparison provider.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SavingsAlternative {
    /// Blacksmith-hosted runners.
    Blacksmith,
    /// GitHub-hosted runners.
    Github,
}

/// Starts and elapsed fractional runtime at one execution place.
#[derive(Serialize, ToSchema)]
pub struct SavingsRuntime {
    /// Runner-taken starts as a decimal string.
    pub starts: String,
    /// Exact elapsed minutes as a decimal string, without invoice rounding.
    pub minutes: String,
}

/// One UTC date for the starts chart and runtime details.
#[derive(Serialize, ToSchema)]
pub struct SavingsDay {
    /// Included UTC date.
    pub day: NaiveDate,
    /// Own-machine starts and elapsed runtime.
    pub machines: SavingsRuntime,
    /// Boat starts and elapsed runtime.
    pub boat: SavingsRuntime,
}

/// A missing monthly-price or registration fact.
#[derive(Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SavingsCostGap {
    /// Trusted workspace creation time is unavailable.
    WorkspaceStart,
    /// Trusted machine registration time is unavailable.
    MachineRegistration {
        /// Own machine that needs trusted registration history.
        machine_id: String,
    },
    /// A registered UTC date has no monthly machine cost.
    MachinePrice {
        /// Own machine needing a monthly cost.
        machine_id: String,
        /// UTC date without a known cost.
        day: NaiveDate,
    },
    /// A UTC date has no confirmed monthly plan price.
    PlanPrice {
        /// UTC date without a known plan price.
        day: NaiveDate,
    },
}

/// Why an alternative has no complete cost for this actual VM size.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum SavingsPriceGapReason {
    /// No published size covers both CPU and memory.
    Unavailable,
    /// Several smallest fitting sizes cannot be ordered by both resources.
    Ambiguous,
}

/// An actual job size missing a unique price from the supplied catalog.
#[derive(Serialize, ToSchema)]
pub struct SavingsPriceGap {
    /// Actual runner resources.
    pub size: JobVmSize,
    /// Missing or ambiguous match.
    pub reason: SavingsPriceGapReason,
}

/// Alternative cost and signed savings using its dated USD price list.
#[derive(Serialize, ToSchema)]
pub struct SavingsComparison {
    /// Alternative provider.
    pub alternative: SavingsAlternative,
    /// Date of the exact price list used.
    pub price_list_date: NaiveDate,
    /// Total comparison cost, absent when any used size has no unique price.
    pub cost_cents: Option<Cents>,
    /// Alternative cost minus ciproxy cost, negative when ciproxy costs more.
    pub savings_cents: Option<Cents>,
    /// Actual sizes with no unique comparison price.
    pub missing_prices: Vec<SavingsPriceGap>,
}

/// Workspace-wide report independent of the plan's visible job history.
#[derive(Serialize, ToSchema)]
pub struct SavingsPage {
    /// Selected range.
    pub range: SavingsRange,
    /// First included UTC date.
    pub from: NaiveDate,
    /// First excluded UTC date.
    pub until: NaiveDate,
    /// Own-machine starts and minutes.
    pub machines: SavingsRuntime,
    /// Boat starts and minutes.
    pub boat: SavingsRuntime,
    /// Every UTC date in the range, including dates with zero starts.
    pub days: Vec<SavingsDay>,
    /// Known own-machine registered-day costs in integer US cents.
    pub known_machine_cents: Cents,
    /// Measured Boat cost in integer US cents, including idle sandboxes.
    pub boat_cents: Cents,
    /// Known plan and add-on date costs in integer US cents.
    pub known_plan_cents: Cents,
    /// Complete ciproxy cost, absent when monthly histories are incomplete.
    pub total_cents: Option<Cents>,
    /// Missing facts that prevent a complete ciproxy total.
    pub missing_costs: Vec<SavingsCostGap>,
    /// Both dated alternatives and their signed savings.
    pub comparisons: Vec<SavingsComparison>,
    /// Cheaper alternative when both comparison costs are known.
    pub lead: Option<SavingsAlternative>,
    /// Current caller may enter a machine's monthly cost.
    pub can_add_machine_costs: bool,
    /// False when another refresh raced and the client should read again.
    pub refresh_complete: bool,
    /// One shared page banner.
    pub banner: Option<PageBanner>,
}
