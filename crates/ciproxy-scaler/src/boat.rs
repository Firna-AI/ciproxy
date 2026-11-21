//! Caller-supplied Boat catalog, spend, and refusal facts.

use std::time::Duration;

use ciproxy_protocol::resources::VmSize;

use crate::snapshot::Runner;

/// An opaque size identifier from the Boat catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatSizeId(#[doc = "The opaque size identifier from the backend."] pub String);

/// The identifier of one Boat sandbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxId(#[doc = "The opaque sandbox identifier from the backend."] pub String);

/// A size that the account currently lists as startable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatSize {
    /// Provider identifier used by the start API.
    pub id: BoatSizeId,
    /// Real dimensions supplied by the backend catalog.
    pub size: VmSize,
    /// Comparable catalog price in the caller's common price units.
    pub price: u64,
}

/// Spend and limit for the current UTC calendar month in common money units.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonthlySpend {
    /// Cost of ciproxy sandboxes only, excluding other account use.
    pub spent: u64,
    /// Saved monthly limit, expressed in the same units as `spent`.
    pub limit: u64,
}

/// Backend refusal facts retained until the caller observes recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Refusal {
    /// A transient refusal blocks new starts until the supplied retry time.
    RetryAfter {
        /// Provider-supplied time, on the same clock as the snapshot.
        not_before: Duration,
    },
    /// Starts wait for a newly accepted key.
    RejectedKey,
    /// Starts wait for the backend to confirm billing recovery.
    BillingRequired,
    /// This catalog size is unavailable on the account's current plan.
    SizeNotOnPlan(BoatSizeId),
}

/// Facts needed to choose a Boat size, with no key or backend dependency.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatAccount {
    /// Workspace setting that allows new overflow starts.
    pub enabled: bool,
    /// Whether the customer currently has a connected, accepted key.
    pub connected: bool,
    /// Spend in the current UTC calendar month.
    pub spend: MonthlySpend,
    /// Current catalog, with equal-price choices in caller preference order.
    pub sizes: Vec<BoatSize>,
    /// Largest available size, selected by the backend's catalog order.
    ///
    /// The caller excludes sizes refused by the current plan. The scaler
    /// does not invent an ordering for incomparable resource dimensions.
    pub largest_size: Option<BoatSizeId>,
    /// Refusals that still apply to this account or catalog.
    pub refusals: Vec<Refusal>,
}

/// A listed sandbox, including sandboxes that ciproxy must never change.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sandbox {
    /// Provider identifier used for deletion.
    pub id: SandboxId,
    /// Whether this sandbox carries this workspace's ciproxy mark.
    pub marked: bool,
    /// Current runner, or confirmed absence of both runner and running job.
    pub runner: Option<BoatRunner>,
}

/// A retained Boat runner and the complete requirement saved at its start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoatRunner {
    /// Immutable original request, actual resources, and current lifecycle.
    pub runner: Runner,
    /// Saved StartRecord requirement for the runner's original image release.
    pub required_root_disk_bytes: u64,
}
