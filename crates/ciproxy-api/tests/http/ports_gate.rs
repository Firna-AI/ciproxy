//! Explicit readiness signals for organization-lease integration scenarios.

use tokio::sync::Notify;

#[derive(Default)]
pub(super) struct ReadGate {
    pub(super) entered: Notify,
    pub(super) release: Notify,
}
