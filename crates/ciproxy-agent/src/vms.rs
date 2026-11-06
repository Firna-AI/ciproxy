//! Separate systemd units own the jailed VMs across agent process restarts.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::runners::RunnerReport;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::boundaries::Vms;
use crate::error::{Error, ErrorResultExt, Result};
use crate::host_tools::Tools;
use crate::layout::{Layout, jail_uid, unit_name};
use crate::network::{Network, network_name};
use crate::process::{Commands, UnitState, arguments};
use crate::state::VmRecord;
use crate::vm_files::VmFiles;
use crate::vsock::Guests;

pub(crate) struct LinuxVms {
    pub(crate) layout: Layout,
    pub(crate) tools: Arc<dyn Tools>,
    pub(crate) commands: Arc<dyn Commands>,
    pub(crate) network: Arc<dyn Network>,
    pub(crate) files: Arc<dyn VmFiles>,
    pub(crate) guests: Arc<dyn Guests>,
    pub(crate) operation: Mutex<()>,
}

#[async_trait]
impl Vms for LinuxVms {
    async fn configuration_lost(&self, vm: &VmRecord) -> Result<bool> {
        self.guests.configuration_lost(vm).await
    }
    async fn clear_docker(&self, vm: &VmRecord) -> Result<()> {
        self.switch_docker(vm, "/docker-slot").await
    }
    async fn swap_docker(&self, vm: &VmRecord) -> Result<()> {
        self.switch_docker(vm, "/docker-job.ext4").await
    }
    fn allocate(&self, report: RunnerReport, used: &[VmRecord]) -> Result<VmRecord> {
        let network_index = (1..16_383)
            .find(|index| !used.iter().any(|vm| vm.network_index == *index))
            .ok_or(Error::Capacity)?;
        Ok(VmRecord {
            preparation: crate::state::PreparationState::Waiting,
            cache_permissions: None,
            job_facts: None,
            local_id: Uuid::new_v4().simple().to_string(),
            network_index,
            report,
        })
    }

    async fn start(&self, vm: &VmRecord, config: AgentToGuest) -> Result<()> {
        self.tools.prepare().await?;
        {
            let _operation = self.operation.lock().await;
            self.files.prepare(vm).await?;
            self.network.create(vm).await?;
            self.commands
                .run("systemd-run", &self.start_arguments(vm), &[])
                .await?;
        }
        self.guests.attach(vm, Some(config)).await
    }

    async fn restore(&self, vm: &VmRecord) -> Result<()> {
        self.guests.attach(vm, None).await
    }

    async fn request_stop(&self, vm: &VmRecord) -> Result<()> {
        self.guests.send(vm, &AgentToGuest::Shutdown).await
    }

    async fn stop(&self, vm: &VmRecord) -> Result<()> {
        let _operation = self.operation.lock().await;
        self.guests.detach(vm).await?;
        let unit = unit_name(vm);
        if self.commands.unit(&unit).await? == UnitState::Active {
            self.commands
                .run("systemctl", &arguments(&["stop", &unit]), &[])
                .await?;
        }
        self.network.remove(vm).await?;
        self.files.remove(vm)?;
        Ok(())
    }

    async fn send(&self, vm: &VmRecord, message: &AgentToGuest) -> Result<()> {
        self.guests.send(vm, message).await
    }
}

impl LinuxVms {
    fn start_arguments(&self, vm: &VmRecord) -> Vec<String> {
        let uid = jail_uid(vm).to_string();
        arguments(&[
            "--quiet",
            "--collect",
            "--service-type=exec",
            "--unit",
            &unit_name(vm),
            "--property=Delegate=yes",
            "--property=KillMode=control-group",
            "--property=StandardOutput=null",
            "--property=StandardError=null",
            &format!(
                "--property=RequiresMountsFor={}",
                self.layout.data.display()
            ),
            &self.layout.jailer.to_string_lossy(),
            "--id",
            &vm.local_id,
            "--exec-file",
            &self.layout.firecracker.to_string_lossy(),
            "--uid",
            &uid,
            "--gid",
            &uid,
            "--cgroup-version",
            "2",
            "--chroot-base-dir",
            &self.layout.data.join("jailer").to_string_lossy(),
            "--netns",
            &format!("/run/netns/{}", network_name(vm.network_index)),
            "--",
            "--config-file",
            "/config.json",
            "--api-sock",
            "/api.sock",
        ])
    }
}

#[cfg(test)]
#[path = "_tests_/vms_tests.rs"]
mod vms_tests;

impl LinuxVms {
    async fn switch_docker(&self, vm: &VmRecord, path: &str) -> Result<()> {
        let client = reqwest::Client::builder()
            .unix_socket(self.layout.jail_root(vm).join("api.sock"))
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("connect jailed VMM API")?;
        let response = client
            .patch("http://localhost/drives/docker")
            .json(&serde_json::json!({"drive_id":"docker","path_on_host":path}))
            .send()
            .await
            .context("switch checked Docker drive")?;
        if !response.status().is_success() {
            return Err(Error::GuestMismatch);
        }
        Ok(())
    }
}
