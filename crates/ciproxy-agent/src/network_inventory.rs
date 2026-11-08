//! Typed host inventory distinguishes occupied indices from failed inspection.

use std::collections::BTreeSet;
use std::net::IpAddr;

use serde::Deserialize;

use crate::error::{ErrorResultExt, Result};
use crate::network::{LinuxNetwork, host_address, network_name, uplink_address};
use crate::process::arguments;

/// Named agent resources and addresses from one checked host inspection.
pub(crate) struct NetworkInventory {
    /// Namespace names read with typed filesystem errors.
    pub(crate) namespaces: BTreeSet<String>,
    /// Root namespace interfaces, including an unmoved veth peer.
    pub(crate) links: BTreeSet<String>,
    /// Root inet tables, without adopting other nftables families.
    pub(crate) tables: BTreeSet<String>,
    /// Reserved IPv4 filter chains, including forwarding-only orphans.
    pub(crate) forwarding: BTreeSet<String>,
    addresses: BTreeSet<String>,
}

impl NetworkInventory {
    /// Find only canonical names reserved by the agent.
    pub(crate) fn owned(&self) -> Vec<u16> {
        self.namespaces
            .iter()
            .chain(&self.tables)
            .chain(&self.forwarding)
            .filter_map(|name| index(name))
            .chain(
                self.links
                    .iter()
                    .filter_map(|name| index(name.strip_suffix('p').unwrap_or(name))),
            )
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Address overlap alone does not grant deletion ownership.
    pub(crate) fn resources(&self, index: u16) -> bool {
        let name = network_name(index);
        self.namespaces.contains(&name)
            || self.links.contains(&name)
            || self.links.contains(&format!("{name}p"))
            || self.tables.contains(&name)
            || self.forwarding.contains(&name)
    }

    /// Avoid both named resources and conflicting host addresses at allocation.
    pub(crate) fn occupied(&self, index: u16) -> bool {
        self.resources(index)
            || self.addresses.contains(&host_address(index))
            || self.addresses.contains(&uplink_address(index))
    }
}

fn index(name: &str) -> Option<u16> {
    let value = name.strip_prefix("cpx")?.parse::<u16>().ok()?;
    ((1..16_383).contains(&value) && network_name(value) == name).then_some(value)
}

#[derive(Deserialize)]
struct Interface {
    ifname: String,
    #[serde(default)]
    addr_info: Vec<Address>,
}

#[derive(Deserialize)]
struct Address {
    local: IpAddr,
}

#[derive(Deserialize)]
struct Tables {
    nftables: Vec<NftObject>,
}

#[derive(Deserialize)]
struct NftObject {
    table: Option<Table>,
}

#[derive(Deserialize)]
struct Table {
    family: String,
    name: String,
}

impl LinuxNetwork {
    /// Require complete successful inspection before proving presence or absence.
    pub(crate) async fn inventory(&self) -> Result<NetworkInventory> {
        let namespaces = self.commands.namespaces().await?;
        let interfaces = self
            .commands
            .run("ip", &arguments(&["-j", "address", "show"]), &[])
            .await?;
        let tables = self
            .commands
            .run("nft", &arguments(&["-j", "list", "tables"]), &[])
            .await?;
        let interfaces: Vec<Interface> =
            serde_json::from_slice(&interfaces).context("read network links and addresses")?;
        let tables: Tables = serde_json::from_slice(&tables).context("read network tables")?;
        Ok(NetworkInventory {
            namespaces: namespaces.into_iter().collect(),
            links: interfaces
                .iter()
                .map(|interface| interface.ifname.clone())
                .collect(),
            addresses: interfaces
                .into_iter()
                .flat_map(|interface| interface.addr_info)
                .map(|address| address.local.to_string())
                .collect(),
            tables: tables
                .nftables
                .into_iter()
                .filter_map(|entry| entry.table)
                .filter(|table| table.family == "inet")
                .map(|table| table.name)
                .collect(),
            forwarding: self.forwarding_inventory().await?,
        })
    }
}
