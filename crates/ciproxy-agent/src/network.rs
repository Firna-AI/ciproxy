//! One network namespace for each VM with IPv4 egress and an isolated cache port.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::{Error, Result};
use crate::layout::jail_uid;
use crate::network_rules::{namespace_rules, root_rules};
use crate::process::{Commands, arguments};
use crate::state::VmRecord;

#[cfg_attr(test, unimock::unimock(api = NetworkMock))]
#[async_trait]
pub(crate) trait Network: Send + Sync {
    /// Find indices from the agent's reserved host resource names.
    async fn owned(&self) -> Result<Vec<u16>>;
    /// Select an index absent from durable ownership and host resources.
    async fn allocate(&self, used: &[u16]) -> Result<u16>;
    async fn create(&self, vm: &VmRecord) -> Result<()>;
    /// Remove only this index's named resources and confirm their absence.
    async fn remove(&self, index: u16) -> Result<()>;
}

pub(crate) struct LinuxNetwork {
    pub(crate) commands: Arc<dyn Commands>,
}

#[async_trait]
impl Network for LinuxNetwork {
    async fn owned(&self) -> Result<Vec<u16>> {
        Ok(self.inventory().await?.owned())
    }

    async fn allocate(&self, used: &[u16]) -> Result<u16> {
        let inventory = self.inventory().await?;
        (1..16_383)
            .find(|index| !used.contains(index) && !inventory.occupied(*index))
            .ok_or(Error::Capacity)
    }

    async fn create(&self, vm: &VmRecord) -> Result<()> {
        let id = network_name(vm.network_index);
        let peer = format!("{id}p");
        let host = host_address(vm.network_index);
        let guest = uplink_address(vm.network_index);
        let uid = jail_uid(vm).to_string();
        self.commands
            .run("ip", &arguments(&["netns", "add", &id]), &[])
            .await?;
        self.commands
            .run(
                "ip",
                &arguments(&["link", "add", &id, "type", "veth", "peer", "name", &peer]),
                &[],
            )
            .await?;
        self.commands
            .run("ip", &arguments(&["link", "set", &peer, "netns", &id]), &[])
            .await?;
        self.commands
            .run(
                "ip",
                &arguments(&["addr", "add", &format!("{host}/30"), "dev", &id]),
                &[],
            )
            .await?;
        self.commands
            .run("ip", &arguments(&["link", "set", &id, "up"]), &[])
            .await?;
        for args in [
            vec!["link", "set", &peer, "name", "uplink"],
            vec!["addr", "add", &format!("{guest}/30"), "dev", "uplink"],
            vec!["link", "set", "uplink", "up"],
            vec!["link", "set", "lo", "up"],
            vec!["route", "add", "default", "via", &host],
            vec!["tuntap", "add", "dev", "tap0", "mode", "tap", "user", &uid],
            vec!["addr", "add", "172.30.0.1/30", "dev", "tap0"],
            vec!["link", "set", "tap0", "up"],
        ] {
            let mut command = arguments(&["-n", &id]);
            command.extend(arguments(&args));
            self.commands.run("ip", &command, &[]).await?;
        }
        self.commands
            .run(
                "ip",
                &arguments(&[
                    "netns",
                    "exec",
                    &id,
                    "sysctl",
                    "-qw",
                    "net.ipv4.ip_forward=1",
                ]),
                &[],
            )
            .await?;
        self.commands
            .run(
                "ip",
                &arguments(&["netns", "exec", &id, "nft", "-f", "-"]),
                namespace_rules(vm.network_index).as_bytes(),
            )
            .await?;
        self.commands
            .run(
                "nft",
                &arguments(&["-f", "-"]),
                root_rules(vm.network_index).as_bytes(),
            )
            .await?;
        self.commands
            .run("sysctl", &arguments(&["-qw", "net.ipv4.ip_forward=1"]), &[])
            .await?;
        Ok(())
    }

    async fn remove(&self, index: u16) -> Result<()> {
        let id = network_name(index);
        let peer = format!("{id}p");
        let inventory = self.inventory().await?;
        let mut failure = None;
        for (present, program, args) in [
            (
                inventory.namespaces.contains(&id),
                "ip",
                arguments(&["netns", "delete", &id]),
            ),
            (
                inventory.links.contains(&id),
                "ip",
                arguments(&["link", "delete", &id]),
            ),
            (
                inventory.links.contains(&peer),
                "ip",
                arguments(&["link", "delete", &peer]),
            ),
            (
                inventory.tables.contains(&id),
                "nft",
                arguments(&["delete", "table", "inet", &id]),
            ),
        ] {
            if present && let Err(error) = self.commands.run(program, &args, &[]).await {
                tracing::warn!(%error, index, "network deletion failed; checking remaining resources");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
        if self.inventory().await?.resources(index) {
            return Err(failure.unwrap_or(Error::NetworkCleanup { index }));
        }
        Ok(())
    }
}

pub(crate) fn network_name(index: u16) -> String {
    format!("cpx{index}")
}

pub(crate) fn host_address(index: u16) -> String {
    let address = u32::from(index) * 4;
    format!("169.254.{}.{}", address / 256, address % 256 + 1)
}

pub(crate) fn uplink_address(index: u16) -> String {
    let address = u32::from(index) * 4;
    format!("169.254.{}.{}", address / 256, address % 256 + 2)
}
