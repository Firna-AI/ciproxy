//! Install adds only missing host tools, names each repair, and tolerates missing KVM.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::host_packages::{DistributionPackages, Packages};
use crate::host_system::{HostOutput, HostSystemMock};

type Calls = Arc<Mutex<Vec<(String, Vec<String>)>>>;

/// One simulated host. `absent` tools appear after a package-manager install.
struct Host {
    release: &'static str,
    absent: &'static [&'static str],
    fail: Option<(&'static str, &'static str)>,
    cgroup: bool,
    modules_dir: bool,
    /// Whether the package manager supplies the absent tools.
    supplied: bool,
}

const HOST: Host = Host {
    release: "ID=ubuntu\n",
    absent: &[],
    fail: None,
    cgroup: true,
    modules_dir: true,
    supplied: true,
};

fn system(host: Host) -> (Arc<Unimock>, Calls) {
    let installed = Arc::new(AtomicBool::new(false));
    let calls: Calls = Arc::new(Mutex::new(Vec::new()));
    let (seen, recorded) = (installed.clone(), calls.clone());
    let Host {
        release,
        absent,
        fail,
        cgroup,
        modules_dir,
        supplied,
    } = host;
    let mock = Unimock::new((
        HostSystemMock::read
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, path| {
                let present = match path {
                    "/etc/os-release" => return Ok(release.as_bytes().to_vec()),
                    "/proc/sys/kernel/osrelease" => return Ok(b"7.3.4-arch1-2\n".to_vec()),
                    "/sys/fs/cgroup/cgroup.controllers" => cgroup,
                    "/lib/modules/7.3.4-arch1-2/modules.dep" => modules_dir,
                    _ => !absent.contains(&"ca") || seen.load(Ordering::SeqCst),
                };
                if present {
                    Ok(b"present".to_vec())
                } else {
                    Err(Error::Disconnected)
                }
            }))
            .at_least_times(0),
        HostSystemMock::execute
            .each_call(matching!(_, _))
            .answers_arc(Arc::new(move |_, program, args| {
                recorded
                    .lock()
                    .unwrap()
                    .push((program.to_owned(), args.to_vec()));
                let last = args.last().map(String::as_str).unwrap_or_default();
                let failed = match program {
                    "sh" => {
                        absent.contains(&last) && !(supplied && installed.load(Ordering::SeqCst))
                    }
                    "modprobe" => last == "kvm" || fail == Some(("modprobe", last)),
                    _ => fail.is_some_and(|(failing, _)| failing == program),
                };
                let install = args.first().map(String::as_str) != Some("update");
                if !failed && install && matches!(program, "apt-get" | "dnf" | "zypper" | "pacman")
                {
                    installed.store(true, Ordering::SeqCst);
                }
                Ok(HostOutput {
                    code: i32::from(failed),
                    stdout: Vec::new(),
                })
            })),
    ));
    (Arc::new(mock), calls)
}

async fn prepare(host: Host) -> (Result<(), Error>, Vec<(String, Vec<String>)>) {
    let (mock, calls) = system(host);
    let result = DistributionPackages { system: mock }.prepare().await;
    let calls = calls.lock().unwrap().clone();
    (result, calls)
}

fn managed(calls: &[(String, Vec<String>)]) -> Vec<String> {
    calls
        .iter()
        .filter(|(program, _)| matches!(program.as_str(), "apt-get" | "dnf" | "zypper" | "pacman"))
        .map(|(program, args)| format!("{program} {}", args.join(" ")))
        .collect()
}

#[tokio::test]
async fn each_distribution_installs_only_missing_tools_and_tolerates_missing_kvm() {
    let absent: &[&str] = &["ip", "sysctl", "mkfs.xfs", "xfs_info", "ca"];
    for (release, expected) in [
        (
            "ID=debian\n",
            vec![
                "apt-get update",
                "apt-get install -y --no-upgrade iproute2 procps xfsprogs ca-certificates",
            ],
        ),
        (
            "ID=fedora\n",
            vec!["dnf install -y iproute procps-ng xfsprogs ca-certificates"],
        ),
        (
            "ID=\"centos\"\nID_LIKE=\"rhel fedora\"\n",
            vec!["dnf install -y iproute procps-ng xfsprogs ca-certificates"],
        ),
        (
            "ID=\"opensuse-leap\"\n",
            vec![
                "zypper --non-interactive install --no-recommends iproute2 procps xfsprogs ca-certificates",
            ],
        ),
        (
            "ID=arch\n",
            vec!["pacman -S --needed --noconfirm iproute2 procps-ng xfsprogs ca-certificates"],
        ),
    ] {
        let (result, calls) = prepare(Host {
            release,
            absent,
            ..HOST
        })
        .await;
        result.unwrap();
        assert_eq!(managed(&calls), expected, "{release}");
        assert!(!managed(&calls).concat().contains("coreutils"), "{release}");
        for module in ["loop", "xfs", "kvm"] {
            assert!(
                calls
                    .iter()
                    .any(|(program, args)| program == "modprobe" && args == &[module])
            );
        }
    }
}

#[tokio::test]
async fn present_tools_need_no_package_manager_or_known_distribution() {
    let (result, calls) = prepare(Host {
        release: "ID=alpine\n",
        ..HOST
    })
    .await;
    result.unwrap();
    assert!(managed(&calls).is_empty());
    let (result, _) = prepare(Host {
        release: "ID=alpine\n",
        absent: &["nft"],
        ..HOST
    })
    .await;
    assert!(
        matches!(result, Err(Error::Preparation { component: "host tools", repair }) if repair.contains("nft"))
    );
}

#[tokio::test]
async fn preparation_failures_name_the_component_and_the_repair() {
    let absent: &[&str] = &["iptables"];
    for (host, component, action) in [
        (
            Host {
                absent,
                fail: Some(("apt-get", "")),
                ..HOST
            },
            "apt-get",
            "apt-get update",
        ),
        (
            Host {
                release: "ID=arch\n",
                absent,
                fail: Some(("pacman", "")),
                ..HOST
            },
            "pacman",
            "pacman -Syu",
        ),
        (
            Host {
                cgroup: false,
                ..HOST
            },
            "cgroup v2",
            "unified cgroup",
        ),
        (
            Host {
                fail: Some(("modprobe", "xfs")),
                ..HOST
            },
            "XFS kernel module",
            "modprobe xfs",
        ),
        (
            Host {
                fail: Some(("modprobe", "loop")),
                modules_dir: false,
                ..HOST
            },
            "loop kernel module",
            "Reboot into the installed kernel",
        ),
    ] {
        let (result, _) = prepare(host).await;
        let error = result.unwrap_err();
        assert!(
            matches!(&error, Error::Preparation { component: actual, repair } if *actual == component && repair.contains(action)),
            "{error}"
        );
        assert!(error.to_string().contains(action));
    }
}

#[tokio::test]
async fn tools_still_missing_after_install_fail_with_a_repair() {
    let (result, calls) = prepare(Host {
        absent: &["nft"],
        supplied: false,
        ..HOST
    })
    .await;
    assert!(matches!(
        result,
        Err(Error::Preparation {
            component: "host tools",
            ..
        })
    ));
    assert_eq!(managed(&calls).len(), 2);
}
