//! Owned IPv4 filter chains allow VM traffic without changing foreign policies.

use std::collections::BTreeSet;

use crate::error::{ErrorResultExt, Result};
use crate::network::{LinuxNetwork, network_name, uplink_address};
use crate::process::arguments;

const PROGRAMS: &str = "for tool in iptables-nft iptables-legacy; do if command -v \"$tool\" >/dev/null 2>&1; then printf '%s\\n' \"$tool\"; fi; done";

fn chain(index: u16) -> String {
    format!("CPX{index}")
}

fn owned_chains(text: &str) -> BTreeSet<u16> {
    text.lines()
        .filter_map(|line| line.strip_prefix("-N CPX"))
        .filter_map(|value| value.parse::<u16>().ok())
        .filter(|index| (1..16_383).contains(index))
        .filter(|index| {
            text.lines()
                .any(|line| line == format!("-N {}", chain(*index)))
        })
        .collect()
}

impl LinuxNetwork {
    async fn forwarding_programs(&self) -> Result<Vec<String>> {
        let output = self
            .commands
            .run("sh", &arguments(&["-c", PROGRAMS]), &[])
            .await?;
        let text = String::from_utf8(output).context("read IPv4 filter backends")?;
        let programs = text.lines().map(str::to_owned).collect::<Vec<_>>();
        Ok(if programs.is_empty() {
            vec!["iptables".to_owned()]
        } else {
            programs
        })
    }

    /// Include forwarding-only orphans in allocation and final cleanup checks.
    pub(crate) async fn forwarding_inventory(&self) -> Result<BTreeSet<String>> {
        let mut names = BTreeSet::new();
        for program in self.forwarding_programs().await? {
            let output = self
                .commands
                .run(&program, &arguments(&["--wait", "-S"]), &[])
                .await?;
            let text = String::from_utf8(output).context("inspect owned IPv4 forwarding chains")?;
            names.extend(owned_chains(&text).into_iter().map(network_name));
        }
        Ok(names)
    }

    /// Publish narrow accepts after the owned nftables isolation rules exist.
    pub(crate) async fn install_forwarding(&self, index: u16) -> Result<()> {
        let chain = chain(index);
        let name = network_name(index);
        let uplink = uplink_address(index);
        for program in self.forwarding_programs().await? {
            for args in [
                vec!["--wait", "-N", &chain],
                vec![
                    "--wait", "-A", &chain, "-i", &name, "-s", &uplink, "-j", "ACCEPT",
                ],
                vec![
                    "--wait",
                    "-A",
                    &chain,
                    "-o",
                    &name,
                    "-d",
                    &uplink,
                    "-m",
                    "conntrack",
                    "--ctstate",
                    "ESTABLISHED,RELATED",
                    "-j",
                    "ACCEPT",
                ],
                vec!["--wait", "-I", "FORWARD", "1", "-j", &chain],
            ] {
                self.commands.run(&program, &arguments(&args), &[]).await?;
            }
        }
        Ok(())
    }

    /// Remove only this network's jump and chain in each available backend.
    pub(crate) async fn remove_forwarding(&self, index: u16) -> Result<()> {
        let chain = chain(index);
        for program in self.forwarding_programs().await? {
            let output = self
                .commands
                .run(&program, &arguments(&["--wait", "-S"]), &[])
                .await?;
            let text = String::from_utf8(output).context("inspect IPv4 forwarding cleanup")?;
            if !owned_chains(&text).contains(&index) {
                continue;
            }
            let jump = format!("-A FORWARD -j {chain}");
            for _ in text.lines().filter(|line| *line == jump) {
                self.commands
                    .run(
                        &program,
                        &arguments(&["--wait", "-D", "FORWARD", "-j", &chain]),
                        &[],
                    )
                    .await?;
            }
            self.commands
                .run(&program, &arguments(&["--wait", "-F", &chain]), &[])
                .await?;
            self.commands
                .run(&program, &arguments(&["--wait", "-X", &chain]), &[])
                .await?;
        }
        Ok(())
    }
}
