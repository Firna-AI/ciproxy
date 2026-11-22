//! macOS parts: launchd, Virtualization.framework through `ciproxy-vz`,
//! APFS clones on the agent's own volume, and pf.
//!
//! [The macOS host design](../../macos-host.md) gives the whole design.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{Mutex, mpsc};

use crate::boundaries::{DockerDisks, Host, Service, Updates, Vms};
use crate::error::{Error, Result};
use crate::macos::guests::{HelperLiveness, HelperTransport, VmFolders};
use crate::macos::paths::{AREA, CONTROL};
use crate::preparation::{Preparation, Uninstall};
use crate::vsock::GuestChannels;
use crate::workload_space::WorkloadSpace;

use super::{Machine, MachineInput, Platform, UninstallParts};

mod area;
mod cache;
mod guard;
mod host;
mod images;
mod isolation;
mod launchctl;
mod preparation;
mod recovery;
mod seat_states;
mod service;
mod updates;
mod vm_files;
pub(crate) mod vm_helper;
mod vms;

use cache::NoMachineCache;
use host::MacHost;
use images::MacImages;
use isolation::{Isolation, PfIsolation};
use launchctl::{Launchctl, Launchd};
use preparation::{MacPreparation, MacUninstall};
use seat_states::SeatStates;
use service::LaunchdService;
use updates::MacUpdates;
use vms::MacVms;

/// The bytes that the area volume keeps for images: the newest image, the
/// next one while it downloads, and its archive. Provisional until the
/// first measured image.
const IMAGE_BUDGET_BYTES: u64 = 200 * 1024 * 1024 * 1024;

/// Build saved states of the seats for warm starts. Off: on the test Mac, a
/// save works only in a logged-in user's GUI session, and the root daemon
/// starts helpers in the system domain, where a save stops the VM. Cal
/// decides between cold starts only and warm starts in a login session.
const SAVED_STATES: bool = false;

/// The macOS adapters of one invocation.
pub(crate) struct MacPlatform {
    images: Arc<MacImages>,
    isolation: Arc<dyn Isolation>,
}

impl MacPlatform {
    pub(crate) fn new() -> Self {
        let client = reqwest::Client::builder()
            .https_only(true)
            .connect_timeout(Duration::from_secs(20))
            .read_timeout(Duration::from_secs(120))
            .build()
            .unwrap_or_default();
        Self {
            images: Arc::new(MacImages::new(PathBuf::from(AREA), client)),
            isolation: Arc::new(PfIsolation::new(
                false,
                PathBuf::from(CONTROL).join("pf-token"),
            )),
        }
    }
}

#[async_trait]
impl Platform for MacPlatform {
    type Prepared = ();

    fn service(&self) -> Arc<dyn Service> {
        Arc::new(LaunchdService)
    }

    fn host(&self) -> Arc<dyn Host> {
        Arc::new(MacHost::new(self.images.clone()))
    }

    fn prepare(&self) -> Result<(Arc<dyn Preparation>, ())> {
        let area_bytes = match std::env::var("CIPROXY_AREA_BYTES") {
            Ok(value) => Some(value.trim().parse().map_err(|_| Error::Preparation {
                component: "area size setting",
                repair: "Set CIPROXY_AREA_BYTES to an integer byte count.",
            })?),
            Err(_) => None,
        };
        Ok((
            Arc::new(MacPreparation {
                isolation: self.isolation.clone(),
                launchd: Arc::new(Launchctl),
                area_bytes,
            }),
            (),
        ))
    }

    fn machine(&self, _prepared: (), input: MachineInput<'_>) -> Result<Machine> {
        let folders = VmFolders {
            area: PathBuf::from(AREA),
        };
        let (events, guests) = mpsc::channel(256);
        let channels = Arc::new(GuestChannels::with_parts(
            Arc::new(HelperTransport {
                folders: folders.clone(),
            }),
            Arc::new(HelperLiveness {
                folders: folders.clone(),
            }),
            events,
        ));
        let launchd: Arc<dyn Launchd> = Arc::new(Launchctl);
        let (guard_launchd, guard_folders) = (launchd.clone(), folders.clone());
        let vms: Arc<dyn Vms> = Arc::new(MacVms {
            seats: Arc::new(SeatStates::new(folders.clone(), launchd.clone())),
            folders,
            launchd,
            guests: channels,
            isolation: self.isolation.clone(),
            builds: SAVED_STATES && !input.uninstall,
            operation: Mutex::new(()),
        });
        Ok(Machine {
            image_budget: IMAGE_BUDGET_BYTES,
            images: self.images.clone(),
            cache: Arc::new(NoMachineCache),
            vms,
            guests,
            tasks: Box::new((
                stay_awake(),
                guard::spawn(self.isolation.clone(), guard_launchd, guard_folders),
            )),
        })
    }

    /// Jobs on a Mac have no Docker.
    fn dockers(
        &self,
        _allocation: Arc<Mutex<()>>,
        _space: Arc<dyn WorkloadSpace>,
        _image_budget: u64,
    ) -> Option<Arc<dyn DockerDisks>> {
        None
    }

    /// The machine cache does not run on macOS yet. Jobs use the GitHub cache.
    async fn install_cache(&self) -> Result<()> {
        Ok(())
    }

    fn updates(&self) -> Arc<dyn Updates> {
        Arc::new(MacUpdates {
            release_server: std::env::var("CIPROXY_RELEASE_SERVER_URL")
                .unwrap_or_else(|_| "https://get.ciproxy.dev".to_owned()),
        })
    }

    fn uninstall(&self, parts: UninstallParts) -> Arc<dyn Uninstall> {
        Arc::new(MacUninstall {
            _runtime: parts.runtime,
            images: parts.images,
            preparation: parts.preparation,
            service: parts.service,
            store: parts.store,
            agent: parts.agent,
        })
    }
}

/// Keep the Mac out of idle sleep while this process runs. A sleeping Mac
/// takes no jobs, and its VMs pause. `caffeinate` ends with the agent, and
/// dropping the child ends it too. A failure only logs a warning.
fn stay_awake() -> Option<tokio::process::Child> {
    let child = tokio::process::Command::new("caffeinate")
        .args(["-i", "-w", &std::process::id().to_string()])
        .kill_on_drop(true)
        .spawn();
    match child {
        Ok(child) => Some(child),
        Err(error) => {
            tracing::warn!(%error, "the Mac can sleep while the agent runs");
            None
        }
    }
}

/// Name the agent part that macOS does not support.
pub(super) fn not_supported(part: &'static str) -> Error {
    Error::PlatformUnsupported { part }
}
