//! `ciproxy-agent vm-run LOCAL_ID`: one VM in its own launchd job.
//!
//! The VM lives in this process, so it survives an agent restart and an
//! agent update. The agent reaches it through the control socket. The
//! runner registration never passes through this process: `CONNECT` hands
//! the vsock stream fd to the agent.

use std::os::fd::OwnedFd;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_vz::{Bundle, StopReason as VzStop, Vm, VmSettings, VmState};
use tokio::signal::unix::{SignalKind, signal};

use crate::error::{Error, ErrorResultExt, Result};
use crate::macos::guests::VmFolders;
use crate::macos::paths::{AREA, valid_local_id};
use crate::macos::seats::StartKind;
use crate::macos::vm_control::{ControlError, StopReason, VmPhase};
use crate::macos::vm_control_server::{VmControl, bind, serve};
use crate::macos::vm_spec::{ExitReason, VmExit, VmSpec};

/// The file that `SAVE` writes in the VM folder.
pub(crate) const SAVED_STATE_FILE: &str = "state.vzvmsave";

/// Run one VM until it stops, then write `exit.json` and return.
pub(crate) async fn run(local_id: &str) -> Result<()> {
    if !valid_local_id(local_id) || rustix::process::geteuid().as_raw() != 0 {
        return Err(helper("the VM helper needs root and a valid local ID"));
    }
    let folders = VmFolders {
        area: PathBuf::from(AREA),
    };
    let folder = folders.folder(local_id);
    let spec: VmSpec =
        serde_json::from_slice(&std::fs::read(folder.join("vm.json")).context("read the VM file")?)
            .context("decode the VM file")?;
    spec.check().map_err(|error| helper(&error.to_string()))?;
    let settings = VmSettings {
        cpus: spec.cpus,
        memory_bytes: spec.memory_bytes,
        display: None,
    };
    let vm = Arc::new(Vm::new(&Bundle::new(&spec.bundle), &settings).map_err(vz)?);
    let listener = bind(&folders.client(local_id).socket).context("bind the control socket")?;
    let mut terminate = signal(SignalKind::terminate()).context("watch SIGTERM")?;
    start(&vm, &spec).await?;
    let control = Arc::new(HelperVm {
        vm: vm.clone(),
        folder: folder.clone(),
    });
    let server = tokio::spawn(serve(listener, control));
    let reason = tokio::select! {
        reason = vm.stopped() => stop_reason(&reason),
        _ = terminate.recv() => {
            if let Err(error) = vm.stop().await {
                tracing::warn!(%error, "VM stop after SIGTERM failed");
            }
            StopReason::HostStop
        }
    };
    tracing::info!(local_id, reason = reason.word(), "VM stopped");
    write_exit(&folder, reason)?;
    server.abort();
    let _ = std::fs::remove_file(folders.client(local_id).socket);
    Ok(())
}

async fn start(vm: &Vm, spec: &VmSpec) -> Result<()> {
    match (spec.start, &spec.saved_state) {
        (StartKind::Cold, _) => vm.start().await.map_err(vz),
        (StartKind::Restore, Some(state)) => {
            vm.restore(state).await.map_err(vz)?;
            vm.resume().await.map_err(vz)
        }
        (StartKind::Restore, None) => Err(helper("a restore needs a state file")),
    }
}

/// Write `exit.json` with a rename, so the agent never reads half a file.
fn write_exit(folder: &Path, reason: StopReason) -> Result<()> {
    let bytes = serde_json::to_vec(&VmExit {
        reason: ExitReason::from(reason),
    })
    .context("encode the exit file")?;
    let staged = folder.join("exit.json.pending");
    std::fs::write(&staged, bytes).context("write the exit file")?;
    std::fs::rename(&staged, folder.join("exit.json")).context("publish the exit file")
}

struct HelperVm {
    vm: Arc<Vm>,
    folder: PathBuf,
}

#[async_trait]
impl VmControl for HelperVm {
    fn phase(&self) -> VmPhase {
        match self.vm.state() {
            VmState::Running => VmPhase::Running,
            VmState::Paused => VmPhase::Paused,
            VmState::Changing => VmPhase::Starting,
            VmState::Stopped | VmState::Error => VmPhase::Stopped,
        }
    }

    async fn connect(&self, port: u32) -> std::result::Result<OwnedFd, ControlError> {
        self.vm.connect(port).await.map_err(|error| match error {
            ciproxy_vz::Error::GuestNotListening { .. } => ControlError::GuestNotListening,
            error => {
                tracing::warn!(%error, port, "vsock connect failed");
                ControlError::Failed
            }
        })
    }

    async fn request_stop(&self) -> std::result::Result<(), ControlError> {
        self.vm.request_stop().map_err(|error| {
            tracing::warn!(%error, "guest stop request failed");
            ControlError::Failed
        })
    }

    async fn save(&self) -> std::result::Result<(), ControlError> {
        let result = async {
            self.vm.pause().await?;
            self.vm.save(&self.folder.join(SAVED_STATE_FILE)).await
        }
        .await;
        result.map_err(|error| {
            tracing::warn!(%error, "VM save failed");
            ControlError::Failed
        })
    }

    async fn stopped(&self) -> StopReason {
        stop_reason(&self.vm.stopped().await)
    }
}

fn stop_reason(reason: &VzStop) -> StopReason {
    match reason {
        VzStop::Guest => StopReason::GuestShutdown,
        VzStop::Host => StopReason::HostStop,
        VzStop::Error(error) => {
            tracing::warn!(error, "VM stopped with an error");
            StopReason::Failure
        }
    }
}

fn vz(error: ciproxy_vz::Error) -> Error {
    helper(&error.to_string())
}

fn helper(reason: &str) -> Error {
    Error::VmHelper {
        reason: reason.to_owned(),
    }
}
