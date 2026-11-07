//! Delegate label binding creation and retirement to the GitHub service.

use uuid::Uuid;

use crate::error::{Result, ServiceResultExt};
use crate::state::ApiState;

pub(crate) async fn cleanup(state: &ApiState, workspace: Uuid) -> Result<()> {
    state.github.reconcile_labels(workspace).await.api()
}
