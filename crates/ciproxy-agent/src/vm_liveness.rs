//! Whether the process that owns a VM still runs. The guest bridge asks
//! this before each connect, so it ends a channel whose VM stopped.
//!
//! Linux asks systemd for the VM unit. macOS asks launchd and the VM
//! helper's control socket.

use async_trait::async_trait;

use crate::error::Result;
use crate::state::VmRecord;
pub(crate) use crate::unit_state::UnitState;

#[cfg_attr(test, unimock::unimock(api = VmLivenessMock))]
#[async_trait]
pub(crate) trait VmLiveness: Send + Sync {
    async fn state(&self, vm: &VmRecord) -> Result<UnitState>;
}
