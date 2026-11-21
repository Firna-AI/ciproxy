//! Linux procfs identifies the child's actual listener without port reuse.

use std::collections::HashSet;
use std::io::ErrorKind;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::path::PathBuf;
use std::time::Duration;

use tokio::process::Child;
use tokio::task::yield_now;
use tokio::time::timeout;

/// Discover a port that the executable already owns, with no bind-and-release gap.
///
/// The child binds `127.0.0.1:0`. Linux procfs lists namespace-wide TCP sockets,
/// so only LISTEN rows whose inode appears in `/proc/<pid>/fd` are candidates.
/// The fixture requires Linux procfs and fails when it is unavailable.
pub(crate) async fn bound_address(child: &mut Child) -> SocketAddr {
    timeout(Duration::from_secs(15), async {
        loop {
            assert_eq!(
                child.try_wait().unwrap(),
                None,
                "the real executable must reach HTTP startup"
            );
            if let Some(address) = listening_address(child.id().unwrap()) {
                return address;
            }
            yield_now().await;
        }
    })
    .await
    .expect("the executable must own a loopback HTTP listener")
}

fn listening_address(pid: u32) -> Option<SocketAddr> {
    let root = PathBuf::from(format!("/proc/{pid}"));
    let sockets: HashSet<_> = std::fs::read_dir(root.join("fd"))
        .expect("the executable fixture requires Linux procfs socket descriptors")
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            match std::fs::read_link(path) {
                Ok(target) => target
                    .to_str()
                    .and_then(|target| target.strip_prefix("socket:["))
                    .and_then(|target| target.strip_suffix(']'))
                    .map(str::to_owned),
                Err(error) if error.kind() == ErrorKind::NotFound => None,
                Err(error) => panic!("read child socket descriptor: {error}"),
            }
        })
        .collect();
    let table = std::fs::read_to_string(root.join("net/tcp"))
        .expect("the executable fixture requires the Linux procfs TCP table");
    let addresses: Vec<_> = table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields[3] != "0A" || !sockets.contains(fields[9]) {
                return None;
            }
            let (ip, port) = fields[1].split_once(':').unwrap();
            let ip = Ipv4Addr::from(u32::from_str_radix(ip, 16).unwrap().to_ne_bytes());
            let port = u16::from_str_radix(port, 16).unwrap();
            ip.is_loopback()
                .then_some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        })
        .collect();
    match addresses.as_slice() {
        [] => None,
        [address] => Some(*address),
        _ => panic!("the executable must own exactly one loopback listener: {addresses:?}"),
    }
}
