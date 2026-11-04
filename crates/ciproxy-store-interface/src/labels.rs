//! Label configuration queries.

use uuid::Uuid;

use crate::error::Result;
use crate::vm_size::VmSize;

/// What to do when no allowed machine or Boat size fits a label.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum WhenNothingFits {
    /// Keep the job queued.
    Wait,
    /// Use the largest Boat size that the account can start.
    LargestBoat,
}

/// A VM size selected by a workflow label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    /// The internal label identifier.
    pub id: Uuid,
    /// The workflow label name.
    pub name: String,
    /// The requested VM resources.
    pub size: VmSize,
    /// Whether own machines may serve the label.
    pub allow_machines: bool,
    /// Whether Boat may serve the label.
    pub allow_boat: bool,
    /// The action when no current resource fits.
    pub when_nothing_fits: WhenNothingFits,
}

/// Queries scoped to the workspace that owns each label.
pub trait LabelStore: Send + Sync {
    /// Insert a label. Label names do not change after insertion.
    fn create_label(&self, workspace_id: Uuid, label: &Label) -> Result<()>;
    /// Read all labels of one workspace.
    fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
    /// Change the size, source switches, and fallback of a label.
    fn update_label(&self, workspace_id: Uuid, label: &Label) -> Result<()>;
}
