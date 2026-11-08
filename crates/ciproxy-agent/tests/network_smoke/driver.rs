//! Opt-in network smoke driver executes the production namespace and VM setup.

#![expect(
    dead_code,
    reason = "The included production modules expose other adapters."
)]

#[path = "../../src/cache_authority.rs"]
mod cache_authority;
#[path = "../../src/error.rs"]
mod error;
#[path = "../../src/host_packages.rs"]
mod host_packages;
#[path = "../../src/host_system.rs"]
mod host_system;
#[path = "../../src/layout.rs"]
mod layout;
#[path = "../../src/network.rs"]
mod network;
#[path = "../../src/network_forwarding.rs"]
mod network_forwarding;
#[path = "../../src/network_inventory.rs"]
mod network_inventory;
#[path = "../../src/network_preparation.rs"]
mod network_preparation;
#[path = "../../src/network_rules.rs"]
mod network_rules;
#[path = "../../src/process.rs"]
mod process;
#[path = "../../src/state.rs"]
mod state;
#[path = "../../src/vm_config.rs"]
mod vm_config;

use std::sync::Arc;

use serde_json::json;

use host_system::LinuxHostSystem;
use network::{LinuxNetwork, Network};
use network_preparation::NetworkPackages;
use process::LinuxCommands;
use state::VmRecord;
use vm_config::VmConfig;

#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let network = LinuxNetwork::new(
        Arc::new(LinuxCommands),
        Arc::new(NetworkPackages {
            system: Arc::new(LinuxHostSystem),
        }),
    );
    match args.first().map(String::as_str) {
        Some("create") => {
            let index = network.allocate(&[]).await.unwrap();
            let record: VmRecord = serde_json::from_value(json!({
                "preparation":"Waiting", "cache_authority":"Waiting", "job_facts":null,
                "local_id":"network-smoke", "network_index":index,
                "report":{
                    "configuration_lost":false, "runner_id":"network-smoke",
                    "organization_id":"0", "label":"ciproxy", "image_id":"network-smoke",
                    "size":{"vcpu":2,"memory_bytes":8000000000u64,"disk_bytes":80899612672u64},
                    "requested_size":{"vcpu":2,"memory_bytes":8000000000u64,"disk_bytes":20000000000u64},
                    "slot_indices":[0,1], "state":{"state":"starting"}
                }
            })).unwrap();
            if let Err(error) = network.create(&record).await {
                let cleanup = network.remove(index).await;
                panic!("network preparation failed: {error}; cleanup: {cleanup:?}");
            }
            println!(
                "{}",
                json!({"index":index,"namespace":network::network_name(index),
                "uid":layout::jail_uid(&record),"configuration":VmConfig::for_vm(&record)})
            );
        }
        Some("remove") => {
            let index = args.get(1).unwrap().parse().unwrap();
            network.remove(index).await.unwrap();
            println!("{}", json!({"removed":true,"index":index}));
        }
        _ => panic!("usage: network-smoke-driver create | remove INDEX"),
    }
}
