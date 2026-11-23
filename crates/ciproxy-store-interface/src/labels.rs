//! Label configuration queries.

use ciproxy_protocol::platform::Platform;
use uuid::Uuid;

use crate::error::Result;
use crate::overflow::provider::OverflowProvider;
use crate::vm_size::VmSize;

/// What to do when no place of a label fits its size.
#[derive(Clone, Copy, Debug, Eq, PartialEq, strum::EnumString, strum::IntoStaticStr)]
#[strum(serialize_all = "snake_case")]
pub enum WhenNothingFits {
    /// Keep the job queued.
    Wait,
    /// Use the largest size of the first eligible overflow provider.
    LargestOverflow,
}

/// One place in a label's order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelPlace {
    /// The workspace's own machines.
    Machine,
    /// One overflow provider.
    Provider(OverflowProvider),
}

impl LabelPlace {
    /// The stored text for the place.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::Provider(provider) => provider.as_str(),
        }
    }
}

/// A VM size selected by a workflow label.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label {
    /// The internal label identifier.
    pub id: Uuid,
    /// The workflow label name.
    pub name: String,
    /// The platform of the label's job VMs. It does not change after insertion.
    pub platform: Platform,
    /// The requested VM resources.
    pub size: VmSize,
    /// The places that may serve the label, in order.
    pub order: Vec<LabelPlace>,
    /// The action when no current resource fits.
    pub when_nothing_fits: WhenNothingFits,
}

/// Queries scoped to the workspace that owns each label.
pub trait LabelStore: Send + Sync {
    /// Insert a label with an explicit order. Its name and platform are fixed.
    fn create_label(&self, workspace_id: Uuid, label: &Label) -> Result<()>;
    /// Insert a label with own machines, followed by offered providers for Linux.
    fn create_label_with_default_order(&self, workspace_id: Uuid, label: &Label) -> Result<()>;
    /// Read all active labels of one workspace with their saved orders.
    fn labels(&self, workspace_id: Uuid) -> Result<Vec<Label>>;
    /// Replace the size, order, and fallback of a label together.
    /// Keep its name and platform.
    fn update_label(&self, workspace_id: Uuid, label: &Label) -> Result<()>;
}
