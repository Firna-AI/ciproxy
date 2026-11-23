//! Find missing host tools and install only their packages with the native manager.

use crate::error::Result;
use crate::host_system::HostSystem;
use crate::process::arguments;

/// How the agent finds one required host tool.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Check {
    /// A command on the root `PATH`.
    Command(&'static str),
    /// One of these files, such as the distribution CA bundle.
    AnyFile(&'static [&'static str]),
}

/// One host tool and the package that supplies it for each manager.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Requirement {
    pub(crate) check: Check,
    /// Package names in the order of [`Manager`]: APT, DNF, Zypper, pacman.
    packages: [&'static str; 4],
}

const fn command(name: &'static str, packages: [&'static str; 4]) -> Requirement {
    Requirement {
        check: Check::Command(name),
        packages,
    }
}

const fn same(name: &'static str, package: &'static str) -> Requirement {
    command(name, [package; 4])
}

const CA_BUNDLES: &[&str] = &[
    "/etc/ssl/certs/ca-certificates.crt",
    "/etc/pki/tls/certs/ca-bundle.crt",
    "/etc/ssl/ca-bundle.pem",
];

/// The tools that VM networking needs. An updated daemon checks only these.
pub(crate) const NETWORK: &[Requirement] = &[
    command("ip", ["iproute2", "iproute", "iproute2", "iproute2"]),
    same("nft", "nftables"),
    same("iptables", "iptables"),
    command("sysctl", ["procps", "procps-ng", "procps", "procps-ng"]),
];

/// Every tool that install needs. Coreutils is part of each base system.
pub(crate) const HOST: &[Requirement] = &[
    NETWORK[0],
    NETWORK[1],
    NETWORK[2],
    NETWORK[3],
    same("modprobe", "kmod"),
    same("mkfs.xfs", "xfsprogs"),
    same("xfs_info", "xfsprogs"),
    same("xfs_growfs", "xfsprogs"),
    same("resize2fs", "e2fsprogs"),
    same("e2fsck", "e2fsprogs"),
    command(
        "losetup",
        ["mount", "util-linux", "util-linux", "util-linux"],
    ),
    same("blkid", "util-linux"),
    command(
        "findmnt",
        [
            "util-linux",
            "util-linux",
            "util-linux-systemd",
            "util-linux",
        ],
    ),
    Requirement {
        check: Check::AnyFile(CA_BUNDLES),
        packages: ["ca-certificates"; 4],
    },
];

/// The package managers that install can drive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Manager {
    Apt,
    Dnf,
    Zypper,
    Pacman,
}

/// A package-manager step that failed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Failure {
    Refresh,
    Install,
}

impl Manager {
    /// Select by `ID`, then by each `ID_LIKE` entry of `/etc/os-release`.
    pub(crate) fn detect(release: &[u8]) -> Option<Self> {
        let text = String::from_utf8_lossy(release);
        let value = |key: &str| {
            text.lines()
                .find_map(|line| line.strip_prefix(key))
                .map(|value| value.trim().trim_matches(['"', '\'']).to_owned())
                .unwrap_or_default()
        };
        let id = value("ID=");
        let like = value("ID_LIKE=");
        std::iter::once(id.as_str())
            .chain(like.split_whitespace())
            .find_map(Self::for_id)
    }

    fn for_id(id: &str) -> Option<Self> {
        match id {
            "debian" | "ubuntu" => Some(Self::Apt),
            "fedora" | "rhel" | "centos" | "rocky" | "almalinux" => Some(Self::Dnf),
            "suse" | "opensuse" | "sles" => Some(Self::Zypper),
            "arch" => Some(Self::Pacman),
            _ if id.starts_with("opensuse-") => Some(Self::Zypper),
            _ => None,
        }
    }

    pub(crate) fn program(self) -> &'static str {
        match self {
            Self::Apt => "apt-get",
            Self::Dnf => "dnf",
            Self::Zypper => "zypper",
            Self::Pacman => "pacman",
        }
    }

    /// The repair for a failed install step. Pacman never refreshes alone.
    pub(crate) fn repair(self) -> &'static str {
        match self {
            Self::Pacman => {
                "Run pacman -Syu to update the package databases and the host. Reboot if the kernel changed. Then rerun install."
            }
            _ => {
                "Restore the distribution package sources and clear package-manager locks. Then rerun install."
            }
        }
    }

    /// Install the packages of the missing tools without a general upgrade.
    pub(crate) async fn install(
        self,
        system: &dyn HostSystem,
        missing: &[Requirement],
    ) -> std::result::Result<(), Failure> {
        if self == Self::Apt {
            run(
                system,
                self.program(),
                arguments(&["update"]),
                Failure::Refresh,
            )
            .await?;
        }
        let mut args = arguments(match self {
            Self::Apt => &["install", "-y", "--no-upgrade"],
            Self::Dnf => &["install", "-y"],
            Self::Zypper => &["--non-interactive", "install", "--no-recommends"],
            Self::Pacman => &["-S", "--needed", "--noconfirm"],
        });
        for requirement in missing {
            let package = requirement.packages[self as usize];
            if !args.iter().any(|argument| argument == package) {
                args.push(package.to_owned());
            }
        }
        run(system, self.program(), args, Failure::Install).await
    }
}

async fn run(
    system: &dyn HostSystem,
    program: &str,
    args: Vec<String>,
    failure: Failure,
) -> std::result::Result<(), Failure> {
    match system.execute(program, &args).await {
        Ok(output) if output.code == 0 => Ok(()),
        _ => Err(failure),
    }
}

/// Return the requirements that the host does not meet now.
pub(crate) async fn missing(
    system: &dyn HostSystem,
    requirements: &[Requirement],
) -> Result<Vec<Requirement>> {
    let mut missing = Vec::new();
    for requirement in requirements {
        let present = match requirement.check {
            Check::Command(name) => {
                let script = ["-c", "command -v \"$1\" >/dev/null", "sh", name];
                system.execute("sh", &arguments(&script)).await?.code == 0
            }
            Check::AnyFile(paths) => paths.iter().any(|path| system.read(path).is_ok()),
        };
        if !present {
            missing.push(*requirement);
        }
    }
    Ok(missing)
}

/// Name the missing tools for the install log.
pub(crate) fn names(missing: &[Requirement]) -> String {
    missing
        .iter()
        .map(|requirement| match requirement.check {
            Check::Command(name) => name,
            Check::AnyFile(_) => "CA bundle",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
#[path = "_tests_/host_requirements_tests.rs"]
mod host_requirements_tests;
