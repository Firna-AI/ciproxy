//! Package-manager inputs and optional KVM support use mocked process outcomes.

use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::host_packages::{DistributionPackages, Packages};
use crate::host_system::{HostOutput, HostSystemMock};

#[tokio::test]
async fn native_package_manager_installs_required_tools_and_tolerates_missing_kvm() {
    for (distribution, manager, network) in [
        ("debian", "apt-get", "iproute2"),
        ("fedora", "dnf", "iproute"),
    ] {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let recorded = calls.clone();
        let mock = Arc::new(Unimock::new((
            HostSystemMock::read
                .some_call(matching!("/etc/os-release"))
                .returns(Ok(format!("ID={distribution}\n").into_bytes())),
            HostSystemMock::read
                .some_call(matching!("/sys/fs/cgroup/cgroup.controllers"))
                .returns(Ok(b"cpu memory io".to_vec())),
            HostSystemMock::execute
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, program, args| {
                    recorded
                        .lock()
                        .unwrap()
                        .push((program.to_owned(), args.to_vec()));
                    Ok(HostOutput {
                        code: if program == "modprobe" && args == ["kvm"] {
                            1
                        } else {
                            0
                        },
                        stdout: Vec::new(),
                    })
                })),
        )));
        DistributionPackages {
            system: mock.clone(),
        }
        .prepare()
        .await
        .unwrap();
        let calls = calls.lock().unwrap();
        let installs = calls
            .iter()
            .filter(|(program, args)| {
                program == manager && args.first().map(String::as_str) == Some("install")
            })
            .collect::<Vec<_>>();
        assert_eq!(installs.len(), 1, "{distribution}");
        for package in [
            "xfsprogs",
            network,
            "nftables",
            "coreutils",
            "e2fsprogs",
            "util-linux",
            "kmod",
            "ca-certificates",
        ] {
            assert!(
                installs[0].1.iter().any(|argument| argument == package),
                "{distribution}: {package}"
            );
        }
        assert_eq!(
            calls
                .iter()
                .filter(|(program, args)| program == "apt-get" && args == &["update"])
                .count(),
            usize::from(manager == "apt-get")
        );
        assert!(
            calls
                .iter()
                .any(|(program, args)| program == "modprobe" && args == &["loop"])
        );
        assert!(
            calls
                .iter()
                .any(|(program, args)| program == "modprobe" && args == &["xfs"])
        );
    }
}

#[tokio::test]
async fn preparation_failures_name_the_component_and_the_repair() {
    for (failure, component, action) in [
        ("apt", "apt-get", "apt-get update"),
        ("cgroup", "cgroup v2", "unified cgroup"),
        ("xfs", "XFS kernel module", "modprobe xfs"),
    ] {
        let mock = Arc::new(Unimock::new((
            HostSystemMock::read
                .some_call(matching!("/etc/os-release"))
                .returns(Ok(b"ID=ubuntu\n".to_vec())),
            HostSystemMock::read
                .some_call(matching!("/sys/fs/cgroup/cgroup.controllers"))
                .answers_arc(Arc::new(move |_, _| {
                    if failure == "cgroup" {
                        Err(Error::Disconnected)
                    } else {
                        Ok(b"cpu memory".to_vec())
                    }
                }))
                .at_least_times(0),
            HostSystemMock::execute
                .each_call(matching!(_, _))
                .answers_arc(Arc::new(move |_, program, args| {
                    let failed = failure == "apt" && program == "apt-get"
                        || failure == "xfs" && program == "modprobe" && args == ["xfs"];
                    Ok(HostOutput {
                        code: if failed { 1 } else { 0 },
                        stdout: Vec::new(),
                    })
                })),
        )));
        let error = DistributionPackages {
            system: mock.clone(),
        }
        .prepare()
        .await
        .unwrap_err();
        assert!(
            matches!(&error, Error::Preparation { component: actual, repair } if *actual == component && repair.contains(action))
        );
        assert!(error.to_string().contains(component));
        assert!(error.to_string().contains(action));
    }
}
