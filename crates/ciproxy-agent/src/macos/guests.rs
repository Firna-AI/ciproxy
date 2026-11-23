//! The guest bridge on macOS: streams and liveness come from the VM helper.

use std::path::{Path, PathBuf};
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::{Instant, sleep, timeout};

use super::vm_control::VmPhase;
use super::vm_control_client::{ControlFailure, VmControlClient};
use super::vm_spec::{ExitReason, VmExit};
use crate::error::{Error, ErrorResultExt, Result};
use crate::guest_transport::{GuestSocket, GuestTransport, framed};
use crate::state::VmRecord;
use crate::unit_state::UnitState;
use crate::vm_liveness::VmLiveness;

/// `ciproxy-guest` listens on this vsock port, as on Linux.
pub(crate) const GUEST_PORT: u32 = 5252;

/// The folders of the VM helpers below the area mount.
#[derive(Clone, Debug)]
pub(crate) struct VmFolders {
    pub(crate) area: PathBuf,
}

impl VmFolders {
    pub(crate) fn folder(&self, local_id: &str) -> PathBuf {
        self.area.join("vms").join(local_id)
    }

    pub(crate) fn client(&self, local_id: &str) -> VmControlClient {
        VmControlClient {
            socket: self.folder(local_id).join("control.sock"),
        }
    }

    pub(crate) fn exit_file(&self, local_id: &str) -> PathBuf {
        self.folder(local_id).join("exit.json")
    }
}

/// The time that a new helper gets to answer on its control socket, through
/// a cold start or a restore of its VM.
pub(crate) const HELPER_START_WAIT: Duration = Duration::from_secs(60);
const HELPER_POLL: Duration = Duration::from_millis(100);

/// Wait until a new helper answers on its control socket.
///
/// `launchctl bootstrap` returns before the helper binds its socket. The
/// guest bridge reads a missing socket with no `exit.json` as a VM that is
/// gone, so a bridge that starts too early ends the new VM at once. A helper
/// that already ended leaves `exit.json`, and this wait fails at once.
pub(crate) async fn wait_for_helper(
    folders: &VmFolders,
    local_id: &str,
    wait: Duration,
) -> Result<()> {
    let client = folders.client(local_id);
    let exit = folders.exit_file(local_id);
    let answered = async {
        loop {
            match client.phase().await {
                Ok(_) => return Ok(()),
                Err(ControlFailure::HelperGone | ControlFailure::TimedOut) if !exit.exists() => {}
                Err(failure) => return Err(Error::from(failure)),
            }
            sleep(HELPER_POLL).await;
        }
    };
    // One deadline for the whole wait, also when a request is in flight.
    timeout(wait, answered).await.map_err(|_| Error::VmHelper {
        reason: "the VM helper did not answer on its control socket".to_owned(),
    })?
}

/// The helper's `ExitTimeOut` of 30 s, then SIGKILL, and a margin.
pub(crate) const HELPER_EXIT_WAIT: Duration = Duration::from_secs(40);

/// Wait until a stopped helper no longer answers on its control socket.
///
/// launchd can drop the job after `launchctl bootout` before the helper
/// process ends. A new VM on the same seat has the same MAC address and
/// machine identifier, so it must not start while the old VM runs. A helper
/// that ended removes its socket, and a killed one leaves a socket that
/// refuses connections.
pub(crate) async fn wait_for_helper_end(
    folders: &VmFolders,
    local_id: &str,
    wait: Duration,
) -> Result<()> {
    let client = folders.client(local_id);
    let deadline = Instant::now() + wait;
    loop {
        if matches!(client.phase().await, Err(ControlFailure::HelperGone)) {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(Error::VmHelper {
                reason: "the VM helper still answers after its stop".to_owned(),
            });
        }
        sleep(HELPER_POLL).await;
    }
}

/// Connect through the helper. A guest that does not listen yet gives
/// `Error::Disconnected`, so the bridge retries after one second.
pub(crate) struct HelperTransport {
    pub(crate) folders: VmFolders,
}

#[async_trait]
impl GuestTransport for HelperTransport {
    async fn connect(&self, vm: &VmRecord) -> Result<Box<dyn GuestSocket>> {
        let stream = self
            .folders
            .client(&vm.local_id)
            .connect(GUEST_PORT)
            .await?;
        Ok(framed(stream))
    }
}

/// A VM lives while its helper answers with a phase other than stopped.
pub(crate) struct HelperLiveness {
    pub(crate) folders: VmFolders,
}

#[async_trait]
impl VmLiveness for HelperLiveness {
    async fn state(&self, vm: &VmRecord) -> Result<UnitState> {
        match self.folders.client(&vm.local_id).phase().await {
            Ok(VmPhase::Stopped) | Err(ControlFailure::HelperGone) => {
                exit_state(&self.folders.exit_file(&vm.local_id))
            }
            Ok(_) => Ok(UnitState::Active),
            Err(failure) => Err(failure.into()),
        }
    }
}

/// A stopped helper leaves `exit.json`. A helper that died before it wrote
/// the file, or a reboot, leaves no file: the VM is absent.
fn exit_state(path: &Path) -> Result<UnitState> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(UnitState::Absent);
        }
        Err(error) => return Err(error).context("read VM exit file"),
    };
    let exit: VmExit = serde_json::from_slice(&bytes).context("decode VM exit file")?;
    Ok(unit_state(exit.reason))
}

/// The runner exit code of each stop reason, as a Firecracker unit gives.
pub(crate) fn unit_state(reason: ExitReason) -> UnitState {
    match reason {
        ExitReason::GuestShutdown => UnitState::Exited(0),
        ExitReason::HostStop => UnitState::Absent,
        ExitReason::Failure => UnitState::Exited(1),
    }
}
