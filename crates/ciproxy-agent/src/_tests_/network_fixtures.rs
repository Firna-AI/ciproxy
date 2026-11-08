//! Host command state models named resources without using the real host.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::json;
use tokio::sync;
use unimock::{Clause, MockFn, Unimock, matching};

use crate::boundaries::StoreMock;
use crate::error::Error;
use crate::host_packages::PackagesMock;
use crate::layout::Layout;
use crate::network::{LinuxNetwork, Network};
use crate::process::{Commands, CommandsMock};
use crate::state::MachineState;
use crate::vms::LinuxVms;

pub(crate) fn prepared_network(commands: Arc<dyn Commands>) -> LinuxNetwork {
    LinuxNetwork::new(
        commands,
        Arc::new(Unimock::new(
            PackagesMock::prepare
                .each_call(matching!())
                .answers(&|_| Ok(())),
        )),
    )
}

#[derive(Clone, Copy, Default)]
pub(crate) enum Deletion {
    #[default]
    Succeed,
    Fail,
    FailAfterRemoval,
}

#[derive(Default)]
pub(crate) struct HostNetworks {
    pub(crate) namespaces: BTreeSet<String>,
    pub(crate) links: BTreeSet<String>,
    pub(crate) tables: BTreeSet<String>,
    pub(crate) forwarding: BTreeSet<String>,
    pub(crate) addresses: Vec<String>,
    pub(crate) deletion: Deletion,
    pub(crate) namespace_failure: bool,
    pub(crate) malformed_tables: bool,
    pub(crate) fail_start: bool,
    pub(crate) deletions: Vec<Vec<String>>,
}

pub(crate) fn commands(host: Arc<Mutex<HostNetworks>>) -> impl Clause {
    let namespaces = host.clone();
    (
        CommandsMock::namespaces.each_call(matching!()).answers_arc(Arc::new(move |_| {
            let host = namespaces.lock().unwrap();
            if host.namespace_failure {
                Err(Error::HostCommand { program: "namespace-inspection".to_owned() })
            } else {
                Ok(host.namespaces.iter().cloned().collect())
            }
        })),
        CommandsMock::run.each_call(matching!(_, _, _)).answers_arc(Arc::new(move |_, program, args, _| {
            let mut host = host.lock().unwrap();
            let words = args.iter().map(String::as_str).collect::<Vec<_>>();
            match (program, words.as_slice()) {
                ("sh", _) => Ok(Vec::new()),
                ("iptables", ["--wait", "-S"]) => Ok(host.forwarding.iter().map(|name| format!("-N {name}\n-A FORWARD -j {name}\n")).collect::<String>().into_bytes()),
                ("iptables", ["--wait", "-N", name]) => { host.forwarding.insert((*name).to_owned()); Ok(Vec::new()) }
                ("iptables", ["--wait", "-X", name]) => {
                    if matches!(host.deletion, Deletion::Fail) { return Err(Error::HostCommand { program:program.to_owned() }); }
                    host.forwarding.remove(*name); Ok(Vec::new())
                }
                ("iptables", _) => Ok(Vec::new()),
                ("ip", ["-j", "address", "show"]) => {
                    let mut links = host.links.iter().map(|name| json!({"ifname":name})).collect::<Vec<_>>();
                    links.push(json!({"ifname":"eth0","addr_info":host.addresses.iter().map(|local| json!({"local":local})).collect::<Vec<_>>()}));
                    Ok(serde_json::to_vec(&links).unwrap())
                }
                ("nft", ["-j", "list", "tables"]) => {
                    if host.malformed_tables { return Ok(b"unknown output".to_vec()); }
                    Ok(serde_json::to_vec(&json!({"nftables":host.tables.iter().map(|name| json!({"table":{"family":"inet","name":name}})).collect::<Vec<_>>()})).unwrap())
                }
                ("ip", ["netns", "delete", name]) | ("ip", ["link", "delete", name]) | ("nft", ["delete", "table", "inet", name]) => {
                    host.deletions.push(args.to_vec());
                    if matches!(host.deletion, Deletion::Fail) {
                        return Err(Error::HostCommand { program:program.to_owned() });
                    }
                    if words[0] == "netns" { host.namespaces.remove(*name); }
                    else if words[0] == "link" { host.links.remove(*name); }
                    else { host.tables.remove(*name); }
                    if matches!(host.deletion, Deletion::FailAfterRemoval) {
                        Err(Error::HostCommand { program:program.to_owned() })
                    } else { Ok(Vec::new()) }
                }
                ("ip", ["netns", "add", name]) => { host.namespaces.insert((*name).to_owned()); Ok(Vec::new()) }
                ("ip", ["link", "add", name, ..]) => { host.links.insert((*name).to_owned()); Ok(Vec::new()) }
                ("nft", ["-f", "-"]) => { let names = host.namespaces.clone(); host.tables.extend(names); Ok(Vec::new()) }
                ("systemd-run", _) if host.fail_start => Err(Error::HostCommand { program:"systemd-run".to_owned() }),
                ("ip" | "nft" | "sysctl" | "systemd-run", _) => Ok(Vec::new()),
                _ => panic!("unexpected host command: {program} {args:?}"),
            }
        })),
    )
}

pub(crate) fn runtime(mock: Arc<Unimock>) -> LinuxVms {
    runtime_with_network(mock.clone(), Arc::new(prepared_network(mock)))
}

pub(crate) fn runtime_with_network(mock: Arc<Unimock>, network: Arc<dyn Network>) -> LinuxVms {
    LinuxVms {
        layout: Layout {
            data: PathBuf::from("/fixture"),
            firecracker: PathBuf::from("/tools/firecracker"),
            jailer: PathBuf::from("/tools/jailer"),
        },
        tools: mock.clone(),
        commands: mock.clone(),
        network,
        files: mock.clone(),
        guests: mock,
        operation: sync::Mutex::new(()),
    }
}

pub(crate) fn saving(bytes: Arc<Mutex<Vec<u8>>>) -> impl Clause {
    StoreMock::save
        .each_call(matching!(_))
        .answers_arc(Arc::new(move |_, state| {
            *bytes.lock().unwrap() = serde_json::to_vec(state).unwrap();
            Ok(())
        }))
}

pub(crate) fn restored(bytes: &Arc<Mutex<Vec<u8>>>) -> MachineState {
    serde_json::from_slice(&bytes.lock().unwrap()).unwrap()
}
