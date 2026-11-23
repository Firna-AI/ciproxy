//! Caller-supplied provider catalogs, spend, refusals, and retained supply.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::resources::VmSize;

use crate::snapshot::Runner;

/// An opaque identifier assigned to a provider by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderId(&'static str);

impl ProviderId {
    /// Use the caller's stable identifier for this provider.
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }
}

/// An opaque size identifier from one provider's catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSizeId(#[doc = "The opaque size identifier from the provider."] pub String);

/// The identifier of a sandbox within its provider.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxId(#[doc = "The opaque sandbox identifier from the provider."] pub String);

/// A size that the account currently lists as startable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderSize {
    /// Size identifier used by the provider's start API.
    pub id: ProviderSizeId,
    /// Real dimensions supplied by the provider's catalog.
    pub size: VmSize,
    /// Comparable catalog price in the caller's common price units.
    pub price: u64,
    /// Cost bound for the next start, absent when a required cost is unknown.
    pub start_bound: Option<StartBound>,
}

/// Full-lifetime spend bound for a prospective start.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct StartBound {
    /// Amount in the current UTC month.
    pub current_month: u64,
    /// Amount in the next UTC month, if the lifetime crosses the boundary.
    pub next_month: Option<u64>,
}

/// Counted costs and the monthly limit in common money units.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MonthlySpend {
    /// Cost of ciproxy sandboxes only, excluding other account use.
    pub spent: u64,
    /// Saved monthly limit, expressed in the same units as `spent`.
    pub limit: u64,
    /// Counted costs for the next UTC month, absent when unbounded.
    pub next_month_spent: Option<u64>,
}

impl MonthlySpend {
    /// Check the limit in each month the bound affects, with unknown costs preserved.
    pub fn can_cover(self, bound: StartBound) -> Option<bool> {
        if self.spent >= self.limit || bound.current_month > self.limit - self.spent {
            return Some(false);
        }
        let Some(next) = bound.next_month else {
            return Some(true);
        };
        let spent = self.next_month_spent?;
        Some(spent <= self.limit && next <= self.limit - spent)
    }

    pub(crate) fn reserve(&mut self, bound: StartBound) -> bool {
        if self.can_cover(bound) != Some(true) {
            return false;
        }
        self.spent += bound.current_month;
        if let Some(next) = bound.next_month {
            self.next_month_spent = self.next_month_spent.map(|spent| spent + next);
        }
        true
    }
}

/// Provider refusal facts retained until the caller observes recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A transient refusal blocks new starts until the supplied retry time.
    RetryAfter {
        /// Provider-supplied time, on the same clock as the snapshot.
        not_before: Duration,
    },
    /// Starts wait for a newly accepted key.
    RejectedKey,
    /// Starts wait for the provider to confirm billing recovery.
    BillingRequired,
    /// This catalog size is unavailable on the account's current plan.
    SizeNotOnPlan(ProviderSizeId),
}

/// One provider's deployment and account facts for this workspace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderAccount {
    /// Stable provider identity used in label orders and destinations.
    pub id: ProviderId,
    /// Whether the deployment has the settings to start on this provider.
    pub available: bool,
    /// Whether this decision may plan new starts from a completed refresh.
    pub refreshed: bool,
    /// Root sizing for new starts from this provider's current image.
    pub root_sizing: RootSizing,
    /// Current account, including any saved catalog or refusals.
    pub facts: AccountFacts,
}

impl ProviderAccount {
    pub(crate) fn connected_and_available(&self) -> bool {
        self.available && self.facts.connected
    }
}

/// Credential-free catalog and admission facts from a provider account.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountFacts {
    /// Whether the customer has a connected account.
    pub connected: bool,
    /// Spend in the current UTC calendar month.
    pub spend: MonthlySpend,
    /// Current catalog, with equal-price choices in caller preference order.
    pub sizes: Vec<ProviderSize>,
    /// Largest available size, selected by the provider's catalog order.
    ///
    /// The caller excludes sizes refused by the current plan. The scaler
    /// does not invent an ordering for incomparable resource dimensions.
    pub largest_size: Option<ProviderSizeId>,
    /// Refusals that still apply to this account or catalog.
    pub refusals: Vec<Refusal>,
}

/// A listed sandbox, including sandboxes that ciproxy must never change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sandbox {
    /// Provider that owns this sandbox.
    pub provider: ProviderId,
    /// Sandbox identifier used for deletion.
    pub id: SandboxId,
    /// Whether this sandbox carries this workspace's ciproxy mark.
    pub marked: bool,
    /// Current runner, or confirmed absence of both runner and running job.
    pub runner: Option<ProviderRunner>,
}

/// A retained provider runner and the complete requirement saved at its start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderRunner {
    /// Immutable original request, actual resources, and current lifecycle.
    pub runner: Runner,
    /// Saved StartRecord requirement for the runner's original image release.
    pub required_root_disk_bytes: u64,
    /// Recorded guest launch time on the snapshot clock, if known.
    pub launched_at: Option<Duration>,
    /// Whether the delivery still needs budget to dispatch.
    pub dispatch: ProviderDispatch,
}

/// Admission state of a saved provider delivery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderDispatch {
    /// The provider request may already have gone out, or is still being prepared in memory.
    Accepted,
    /// The saved request has not gone out and needs a currently eligible size and budget.
    Unsent(Option<StartBound>),
}
