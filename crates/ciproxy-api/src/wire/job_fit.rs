//! Read-only resource fit for the exact labels of waiting jobs.

use serde::Serialize;
use utoipa::ToSchema;

use super::jobs::JobVmSize;
use super::settings::LabelView;

/// One waiting label's current resource policy for every workspace role.
#[derive(Serialize, ToSchema)]
pub struct WaitingLabelFit {
    /// Exact retained label identity, independent of name reuse.
    pub label_id: String,
    /// Retained label name.
    pub label_name: String,
    /// Visible waiting jobs with this exact label as a decimal string.
    pub waiting_jobs: String,
    /// Current label policy and physical own-machine fit, absent after retirement.
    pub current: Option<LabelView>,
    /// Boat resource fit, distinct from current provider availability.
    pub boat: BoatLabelFit,
    /// No permitted resource choice exists, absent while required facts are unknown.
    pub nothing_fits: Option<bool>,
    /// The current caller may edit this active label.
    pub can_edit: bool,
    /// This notice asks an Admin because the current caller cannot edit the label.
    pub ask_admin: bool,
}

/// Boat resources under the label policy, without claiming a start is available.
#[derive(Serialize, ToSchema)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum BoatLabelFit {
    /// The trusted image sizing, account, or catalog observation is unavailable.
    Unavailable,
    /// This label does not permit Boat.
    Disabled,
    /// No catalog size fits and no permitted largest-size fallback exists.
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
        /// Actual resources, which can be smaller than the requested label.
        size: JobVmSize,
        /// A provider refusal currently excludes this size from starts.
        refused: bool,
    },
}
