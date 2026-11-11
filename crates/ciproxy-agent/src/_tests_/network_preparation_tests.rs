//! Older installations receive only missing network tools and can retry a repair.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::host_system::{HostOutput, HostSystemMock};
use crate::network::{LinuxNetwork, Network};
use crate::network_preparation::NetworkPackages;
use crate::process::CommandsMock;

#[tokio::test]
async fn updated_network_repairs_missing_iptables_and_retries_failed_preparation() {
    let attempts = Arc::new(Mutex::new(Vec::<(String, Vec<String>)>::new()));
    let recorded = attempts.clone();
    let system = Arc::new(Unimock::new((
        HostSystemMock::read
            .each_call(matching!("/etc/os-release"))
            .answers(&|_, _| Ok(b"ID=debian\n".to_vec()))
            .at_least_times(0),
        HostSystemMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, program, args| {
                let mut calls = recorded.lock().unwrap();
                let installs = calls.iter().filter(|(name, _)| name == "apt-get").count();
                let code = if program == "sh" {
                    i32::from(args.last().map(String::as_str) == Some("iptables") && installs < 4)
                } else if args.first().map(String::as_str) == Some("install") {
                    i32::from(installs < 2)
                } else {
                    0
                };
                calls.push((program.to_owned(), args.to_vec()));
                Ok(HostOutput {
                    code,
                    stdout: Vec::new(),
                })
            }))
            .at_least_times(0),
    )));
    let commands = Arc::new(Unimock::new((
        CommandsMock::namespaces
            .each_call(matching!())
            .answers(&|_| Ok(Vec::new())),
        CommandsMock::run
            .each_call(matching!(_, _, _))
            .answers(&|_, program, _, _| {
                Ok(match program {
                    "ip" => b"[]".to_vec(),
                    "nft" => b"{\"nftables\":[]}".to_vec(),
                    _ => Vec::new(),
                })
            }),
    )));
    let network = LinuxNetwork::new(commands, Arc::new(NetworkPackages { system }));
    let failed = network.owned().await;
    assert!(matches!(
        failed,
        Err(Error::Preparation {
            component: "VM network tools",
            ..
        })
    ));
    assert_eq!(network.owned().await.unwrap(), Vec::<u16>::new());
    assert_eq!(network.owned().await.unwrap(), Vec::<u16>::new());
    let calls = attempts.lock().unwrap();
    let package_calls = calls
        .iter()
        .filter(|(program, _)| program == "apt-get")
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(
        package_calls,
        vec![
            ("apt-get".to_owned(), vec!["update".to_owned()]),
            (
                "apt-get".to_owned(),
                vec!["install", "-y", "--no-upgrade", "iptables"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            ),
            ("apt-get".to_owned(), vec!["update".to_owned()]),
            (
                "apt-get".to_owned(),
                vec!["install", "-y", "--no-upgrade", "iptables"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            ),
        ]
    );
    assert_eq!(
        calls.iter().filter(|(program, _)| program == "sh").count(),
        12
    );
}
