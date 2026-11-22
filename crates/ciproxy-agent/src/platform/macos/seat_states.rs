//! Saved states of the VM seats: build, check, and cancel.
//!
//! After a job VM ends, its free seat builds a saved state for the image
//! and the VM settings of that job: a cold boot of an image clone, the
//! guest's first `Hello`, then pause, save, and stop. The state file stays
//! inside the bundle, so the clone for the next job also clones the state.
//! A start on the seat cancels a running build.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::ImageId;
use ciproxy_vz::Bundle;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{Instant, sleep, timeout};
use uuid::Uuid;

use super::launchctl::Launchd;
use super::vm_files::{
    STATE_FILE, clone_with, image_bundle, remove_helper_files, remove_path, seat_folder,
    seat_identity, write_helper_files,
};
use crate::error::{Error, ErrorResultExt, Result};
use crate::guest_transport::framed;
use crate::macos::guests::{GUEST_PORT, VmFolders};
use crate::macos::seats::StartKind;
use crate::macos::seats::{SAVED_STATE_FORMAT, SavedState, SeatNeeds};
use crate::macos::vm_control::ControlError;
use crate::macos::vm_control_client::ControlFailure;
use crate::macos::vm_spec::{VM_SPEC_FORMAT, VmSpec};
use crate::persistence::{atomic_private_write, private_directory};

/// A cold boot reached SSH in 8 s on the test Mac. This leaves room.
const GUEST_WAIT: Duration = Duration::from_secs(180);

/// The image and settings that one build saves.
#[derive(Clone, Debug)]
pub(crate) struct BuildRequest {
    pub(crate) seat: u16,
    pub(crate) image: ImageId,
    pub(crate) cpus: u32,
    pub(crate) memory_bytes: u64,
}

pub(crate) struct SeatStates {
    pub(crate) folders: VmFolders,
    pub(crate) launchd: Arc<dyn Launchd>,
    builds: Mutex<HashMap<u16, (String, JoinHandle<()>)>>,
}

impl SeatStates {
    pub(crate) fn new(folders: VmFolders, launchd: Arc<dyn Launchd>) -> Self {
        Self {
            folders,
            launchd,
            builds: Mutex::new(HashMap::new()),
        }
    }

    /// The seat's saved bundle when its record matches `needs`.
    pub(crate) fn usable(&self, needs: SeatNeeds<'_>) -> Option<Bundle> {
        let current = seat_folder(&self.folders.area, needs.seat, false);
        let bytes = std::fs::read(current.join("saved.json")).ok()?;
        let saved: SavedState = serde_json::from_slice(&bytes).ok()?;
        let bundle = current.join("bundle");
        (saved.usable(needs) && bundle.join(STATE_FILE).is_file()).then(|| Bundle::new(bundle))
    }

    /// Remove the seat's saved state, for example after a failed restore.
    pub(crate) fn discard(&self, seat: u16) -> Result<()> {
        remove_path(&seat_folder(&self.folders.area, seat, false))
    }

    /// Start a build in the background, unless one runs for the seat.
    pub(crate) async fn start_build(self: &Arc<Self>, request: BuildRequest) {
        let mut builds = self.builds.lock().await;
        builds.retain(|_, (_, task)| !task.is_finished());
        if builds.contains_key(&request.seat) {
            return;
        }
        let seat = request.seat;
        let local_id = Uuid::new_v4().simple().to_string();
        let states = self.clone();
        let id = local_id.clone();
        let task = tokio::spawn(async move {
            if let Err(error) = states.build(&id, request).await {
                tracing::warn!(%error, seat, "saved-state build failed; jobs boot cold");
            }
            if let Err(error) = states.clean_build(seat, &id).await {
                tracing::warn!(%error, seat, "saved-state build cleanup will retry at start");
            }
        });
        builds.insert(seat, (local_id, task));
    }

    /// Stop a running build of the seat before a job VM takes the seat.
    pub(crate) async fn cancel(&self, seat: u16) -> Result<()> {
        let build = self.builds.lock().await.remove(&seat);
        if let Some((local_id, task)) = build {
            task.abort();
            let _ = task.await;
            self.clean_build(seat, &local_id).await?;
        }
        Ok(())
    }

    async fn clean_build(&self, seat: u16, local_id: &str) -> Result<()> {
        self.launchd.bootout_vm(local_id).await?;
        remove_helper_files(&self.folders, local_id)?;
        remove_path(&seat_folder(&self.folders.area, seat, true))
    }

    async fn build(&self, local_id: &str, request: BuildRequest) -> Result<()> {
        let area = &self.folders.area;
        let build = seat_folder(area, request.seat, true);
        remove_path(&build)?;
        private_directory(&build)?;
        let identity = seat_identity(area, request.seat)?;
        let mac_address = identity.mac_address.to_string();
        let bundle = build.join("bundle");
        clone_with(image_bundle(area, &request.image), bundle.clone(), identity).await?;
        private_directory(&self.folders.folder(local_id))?;
        let spec = VmSpec {
            format: VM_SPEC_FORMAT,
            seat: request.seat,
            cpus: request.cpus,
            memory_bytes: request.memory_bytes,
            bundle: bundle.clone(),
            start: StartKind::Cold,
            saved_state: None,
        };
        write_helper_files(&self.folders, local_id, &spec)?;
        self.launchd.bootstrap_vm(local_id).await?;
        self.wait_for_hello(local_id).await?;
        let client = self.folders.client(local_id);
        client.save().await?;
        self.launchd.bootout_vm(local_id).await?;
        let saved_file = self.folders.folder(local_id).join(STATE_FILE);
        std::fs::rename(&saved_file, bundle.join(STATE_FILE)).context("keep the saved state")?;
        let saved = SavedState {
            format: SAVED_STATE_FORMAT,
            seat: request.seat,
            image: request.image.0.clone(),
            mac_address,
            cpus: request.cpus,
            memory_bytes: request.memory_bytes,
            agent_version: env!("CARGO_PKG_VERSION").to_owned(),
            saved_at_unix_seconds: now_unix_seconds(),
        };
        atomic_private_write(
            &build.join("saved.json"),
            &serde_json::to_vec(&saved).context("encode the saved-state record")?,
        )?;
        let current = seat_folder(area, request.seat, false);
        remove_path(&current)?;
        std::fs::rename(&build, &current).context("publish the saved state")?;
        tracing::info!(seat = request.seat, image = %request.image.0, "saved state ready");
        Ok(())
    }

    /// The guest is ready when its first frame on vsock is a `Hello`. The
    /// connection then closes with nothing sent, so no runner starts.
    async fn wait_for_hello(&self, local_id: &str) -> Result<()> {
        let client = self.folders.client(local_id);
        let deadline = Instant::now() + GUEST_WAIT;
        let stream = loop {
            match client.connect(GUEST_PORT).await {
                Ok(stream) => break stream,
                Err(ControlFailure::Refused(ControlError::GuestNotListening))
                    if Instant::now() < deadline =>
                {
                    sleep(Duration::from_secs(1)).await;
                }
                Err(failure) => return Err(failure.into()),
            }
        };
        let mut socket = framed(stream);
        let first = timeout(Duration::from_secs(30), socket.receive())
            .await
            .map_err(|_| Error::Disconnected)??;
        if matches!(first, GuestToAgent::Hello { .. }) {
            Ok(())
        } else {
            Err(Error::GuestMismatch)
        }
    }
}

fn now_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |time| time.as_secs())
}
