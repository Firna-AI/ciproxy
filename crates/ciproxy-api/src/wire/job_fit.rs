//! Resource fit and current routing for one label.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::facts::LabelMachineFit;
use super::jobs::JobVmSize;
use super::overflow::OverflowProvider;
use super::settings::{LabelPlace, LabelView};

/// One waiting label's current resource policy for every workspace role.
#[derive(Serialize, ToSchema)]
pub struct WaitingLabelFit {
    /// Exact retained label identity, independent of name reuse.
    pub label_id: String,
    /// Retained label name.
    pub label_name: String,
    /// Visible waiting jobs with this exact label as a decimal string.
    pub waiting_jobs: String,
    /// Current label policy and fit, absent after retirement.
    pub current: Option<LabelView>,
    /// No permitted resource choice exists, absent while required facts are unknown.
    pub nothing_fits: Option<bool>,
    /// The current caller may edit this active label.
    pub can_edit: bool,
    /// This notice asks an Admin because the current caller cannot edit the label.
    pub ask_admin: bool,
}

/// Computed capacity and routing for a label or a label preview.
#[derive(Deserialize, Serialize, ToSchema)]
pub struct LabelFit {
    /// Physical offers from the eligible machines, before current occupancy.
    pub machines: Vec<LabelMachineFit>,
    /// Physical fit and current holds for each provider.
    pub providers: Vec<ProviderLabelFit>,
    /// Physical choices in label order, including a fallback when applicable.
    /// Absent when a missing observation prevents a conclusion.
    pub fitting_places: Option<Vec<LabelPlace>>,
    /// Routing choices after the workspace switch and provider holds.
    pub routing: LabelRouting,
    /// Count of connected, non-draining machines.
    pub eligible_machines: u32,
    /// Machines whose prepared capacity or required root allocation is unavailable.
    pub unavailable_machines: u32,
    /// Count of those machines that fit at least one job.
    pub fitting_machines: u32,
    /// Minimum whole jobs per fitting machine, absent when none fits.
    pub minimum_jobs: Option<String>,
    /// Maximum whole jobs per fitting machine, absent when none fits.
    pub maximum_jobs: Option<String>,
    /// Whether no allowed own machine currently fits this size.
    pub no_machine_fits: bool,
}

/// One provider's fit and current start availability for the label.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct ProviderLabelFit {
    /// Provider that owns the catalog and accounts for these resources.
    pub provider: OverflowProvider,
    /// Account presence from a successful observation, absent when unknown.
    pub connected: Option<bool>,
    /// Physical fit under the label's resource policy.
    pub fit: ProviderSizeFit,
    /// Cheapest fitting size after applying any refusals for individual sizes.
    pub eligible_size: Option<ProviderCatalogSize>,
    /// Largest size in the trusted physical catalog.
    pub largest_size: Option<ProviderCatalogSize>,
    /// Complete required root under the provider's trusted image policy.
    pub required_root_disk_bytes: Option<String>,
    /// A reason the provider cannot take this label now, when one applies.
    pub hold: Option<ProviderHold>,
}

/// One trusted catalog size with its complete actual allocation.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct ProviderCatalogSize {
    /// Actual catalog name.
    pub name: String,
    /// Actual resources, with byte counts as decimal strings.
    pub size: JobVmSize,
}

/// Physical provider resources without a claim that a start is available.
#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ProviderSizeFit {
    /// The trusted image sizing, account, or catalog observation is unavailable.
    Unavailable,
    /// The label order does not include this provider.
    Disabled,
    /// No catalog size fits and no physical fallback is selected here.
    NoSizeFits,
    /// A catalog size covers all requested resources.
    Fitting {
        /// Actual catalog size name.
        size_name: String,
        /// Actual byte-precise catalog resources.
        size: JobVmSize,
        /// A provider refusal currently excludes this size from starts.
        refused: bool,
    },
    /// The label permits the largest size with enough disk for the image.
    LargestFallback {
        /// Actual catalog size name.
        size_name: String,
        /// Actual resources which can be smaller than the requested label.
        size: JobVmSize,
        /// A provider refusal currently excludes this size from starts.
        refused: bool,
    },
}

/// Where a job of the label can run before own-machine occupancy is checked.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LabelRouting {
    /// Places that can serve the label at its requested size, in order.
    Places {
        /// Each place that can take the job.
        places: Vec<LabelPlace>,
    },
    /// A provider's largest size can run the job as a fallback.
    Largest {
        /// First provider that can start the fallback.
        provider: OverflowProvider,
    },
    /// No place can take the job now.
    Waits {
        /// Reason for waiting.
        reason: LabelWaitReason,
    },
}

/// First applicable reason a label waits.
#[derive(Deserialize, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LabelWaitReason {
    /// The customer removed all places from the label.
    EmptyOrder,
    /// No permitted place can fit the request or its fallback.
    NothingFits,
    /// Own-machine or provider resource observations are missing.
    Unavailable,
    /// First eligible place that is blocked by its current provider state.
    Provider {
        /// Provider whose hold explains the wait.
        provider: OverflowProvider,
        /// Reason it cannot take this label now.
        hold: ProviderHold,
    },
}

/// Reason a provider cannot start a job now.
#[derive(Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProviderHold {
    /// The workspace switched overflow off.
    Off,
    /// The workspace has no active account for this provider.
    NotConnected,
    /// The provider is unavailable or its observations failed to refresh.
    Unavailable,
    /// The provider rejected its active key.
    KeyRejected,
    /// The provider limit cannot cover a start for this label.
    SpendLimit,
    /// No safe bound exists for one or more starts.
    CostsUnknown,
    /// The provider refused starts until its retry time.
    RetryAfter,
    /// The provider needs an account billing change.
    BillingRequired,
    /// The provider refused every fitting size or the fallback size.
    SizeRefused,
}
