//! Stub adapters that return a "not supported yet" error from each method.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::images::{ImageDownload, ImageReport};
use ciproxy_protocol::jobs::{JobFacts, JobScope};
use ciproxy_protocol::reports::HardwareReport;
use ciproxy_protocol::runners::RunnerReport;
use tokio::sync::mpsc;

use crate::boundaries::{Cache, GuestEvent, Host, ImageLease, Images, Service, Updates, Vms};
use crate::cache_authority::CheckedRuntime;
use crate::error::Result;
use crate::preparation::{Preparation, Uninstall, UninstallState};
use crate::state::VmRecord;

use super::not_supported as error;

const SERVICE: &str = "the launchd service";
const HOST: &str = "the macOS agent";
const PREPARATION: &str = "host preparation";
const IMAGES: &str = "the macOS image store";
const CACHE: &str = "the machine cache";
const VMS: &str = "the macOS VM backend";
const DOCKER: &str = "Docker disks";
const UPDATES: &str = "agent updates";
const UNINSTALL: &str = "uninstall";

pub(super) struct UnsupportedService;

impl Service for UnsupportedService {
    fn install(&self) -> Result<()> {
        Err(error(SERVICE))
    }
    fn stop(&self) -> Result<()> {
        Err(error(SERVICE))
    }
    fn uninstall(&self) -> Result<()> {
        Err(error(SERVICE))
    }
}

/// The root check fails, so every command stops before it changes the Mac.
pub(super) struct UnsupportedHost;

impl Host for UnsupportedHost {
    fn require_root(&self) -> Result<()> {
        Err(error(HOST))
    }
    fn hostname(&self) -> Result<String> {
        Err(error(HOST))
    }
    fn hardware(&self) -> Result<HardwareReport> {
        Err(error(HOST))
    }
}

pub(super) struct UnsupportedPreparation;

#[async_trait]
impl Preparation for UnsupportedPreparation {
    async fn prepare(&self) -> Result<()> {
        Err(error(PREPARATION))
    }
    async fn verify(&self) -> Result<()> {
        Err(error(PREPARATION))
    }
    async fn remove(&self) -> Result<()> {
        Err(error(PREPARATION))
    }
}

pub(super) struct UnsupportedImages;

#[async_trait]
impl Images for UnsupportedImages {
    fn report(&self) -> Result<ImageReport> {
        Err(error(IMAGES))
    }
    fn recover(&self) -> Result<()> {
        Err(error(IMAGES))
    }
    fn hold_newest(&self) -> Result<Arc<dyn ImageLease>> {
        Err(error(IMAGES))
    }
    async fn fetch(&self, _image: &ImageDownload) -> Result<()> {
        Err(error(IMAGES))
    }
}

pub(super) struct UnsupportedCache;

#[async_trait]
impl Cache for UnsupportedCache {
    async fn refresh_budget(&self, _capacity_bytes: u64) -> Result<()> {
        Err(error(CACHE))
    }
    async fn fallback(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(CACHE))
    }
    async fn budget(&self, _capacity_bytes: u64) -> Result<()> {
        Err(error(CACHE))
    }
    async fn report(&self) -> Result<MachineCacheReport> {
        Err(error(CACHE))
    }
    async fn clear(&self) -> Result<()> {
        Err(error(CACHE))
    }
    async fn release(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(CACHE))
    }
    async fn handoff(&self, _vm: &VmRecord, _job: &JobId, _runtime: CheckedRuntime) -> Result<()> {
        Err(error(CACHE))
    }
    async fn prepare(
        &self,
        _runner: &RunnerId,
        _facts: &JobFacts,
    ) -> Result<Option<(CacheConnection, JobScope, u64)>> {
        Err(error(CACHE))
    }
}

/// A VM backend keeps the sender of guest events for as long as it lives.
pub(super) struct UnsupportedVms {
    pub(super) _events: mpsc::Sender<GuestEvent>,
}

#[async_trait]
impl Vms for UnsupportedVms {
    async fn networks(&self) -> Result<Vec<u16>> {
        Err(error(VMS))
    }
    async fn configuration_lost(&self, _vm: &VmRecord) -> Result<bool> {
        Err(error(VMS))
    }
    async fn clear_docker(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(DOCKER))
    }
    async fn allocate(&self, _report: RunnerReport, _used: &[u16]) -> Result<VmRecord> {
        Err(error(VMS))
    }
    async fn remove_network(&self, _index: u16) -> Result<()> {
        Err(error(VMS))
    }
    async fn start(&self, _vm: &VmRecord, _config: AgentToGuest) -> Result<()> {
        Err(error(VMS))
    }
    async fn reattach(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(VMS))
    }
    async fn request_stop(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(VMS))
    }
    async fn stop(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(VMS))
    }
    async fn send(&self, _vm: &VmRecord, _message: &AgentToGuest) -> Result<()> {
        Err(error(VMS))
    }
    async fn swap_docker(&self, _vm: &VmRecord) -> Result<()> {
        Err(error(DOCKER))
    }
}

pub(super) struct UnsupportedUpdates;

#[async_trait]
impl Updates for UnsupportedUpdates {
    async fn check(&self) -> Result<bool> {
        Err(error(UPDATES))
    }
    async fn restart(&self) -> Result<()> {
        Err(error(UPDATES))
    }
}

pub(super) struct UnsupportedUninstall;

#[async_trait]
impl Uninstall for UnsupportedUninstall {
    async fn start(&self) -> Result<UninstallState> {
        Err(error(UNINSTALL))
    }
    async fn finish(&self) -> Result<()> {
        Err(error(UNINSTALL))
    }
}
