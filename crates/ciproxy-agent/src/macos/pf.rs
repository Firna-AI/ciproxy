//! pf rules that leave each guest the internet only. See `macos-host.md`.
//!
//! The spike proved these rules on macOS 26. Without them, a guest reaches
//! the host, the LAN, and the router. Without the IPv6 rule, a guest
//! reaches the host over IPv6 link-local.

use std::net::Ipv4Addr;

use super::paths::PF_ANCHOR;

/// The machine cache port on the host.
pub(crate) const CACHE_PORT: u16 = 7887;

/// The vmnet NAT network of the guests.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VmNetwork {
    /// The host address on the vmnet bridge.
    pub(crate) gateway: Ipv4Addr,
    pub(crate) prefix: u8,
    /// The bridge interface, such as `bridge100`.
    pub(crate) bridge: String,
}

/// The vmnet defaults when its settings file has no shared network values.
pub(crate) const DEFAULT_GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 64, 1);
pub(crate) const DEFAULT_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
pub(crate) const DEFAULT_BRIDGE: &str = "bridge100";

impl VmNetwork {
    /// Build the network from the vmnet settings. Refuse a mask with holes
    /// and an interface name that is not a bridge.
    pub(crate) fn new(gateway: Ipv4Addr, mask: Ipv4Addr, bridge: &str) -> Option<Self> {
        let bits = u32::from(mask);
        let prefix = bits.leading_ones();
        let digits = bridge.strip_prefix("bridge")?;
        let valid_bridge = !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit());
        (bits.checked_shl(prefix).unwrap_or(0) == 0 && (8..=30).contains(&prefix) && valid_bridge)
            .then(|| Self {
                gateway,
                prefix: prefix as u8,
                bridge: bridge.to_owned(),
            })
    }

    pub(crate) fn subnet(&self) -> String {
        let mask = u32::MAX << (32 - u32::from(self.prefix));
        let network = Ipv4Addr::from(u32::from(self.gateway) & mask);
        format!("{network}/{}", self.prefix)
    }
}

/// The rules of the anchor. `machine_cache` passes guests to the cache port
/// on the gateway. The cache port is closed to every other interface in all
/// cases, apart from `lo0`.
pub(crate) fn rules(network: &VmNetwork, machine_cache: bool) -> String {
    let bridge = &network.bridge;
    let subnet = network.subnet();
    let gateway = network.gateway;
    let cache = if machine_cache {
        format!(
            "pass in quick on {bridge} inet proto tcp from {subnet} to {gateway} port {CACHE_PORT}\n"
        )
    } else {
        String::new()
    };
    format!(
        "# ciproxy job VM isolation v1\n\
         pass in quick on {bridge} inet proto udp from any port 68 to {{ 255.255.255.255, {gateway} }} port 67\n\
         pass in quick on {bridge} inet proto {{ udp tcp }} from {subnet} to {gateway} port 53\n\
         {cache}\
         block drop in quick on {bridge} inet6 all\n\
         block drop in quick inet from {subnet} to (self)\n\
         block drop in quick inet from {subnet} to {subnet}\n\
         block drop in quick inet from {subnet} to {{ 10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16, 169.254.0.0/16, 100.64.0.0/10, 127.0.0.0/8, 224.0.0.0/4, 240.0.0.0/4 }}\n\
         pass in quick on lo0 proto tcp from any to any port {CACHE_PORT}\n\
         block drop in quick proto tcp from any to any port {CACHE_PORT}\n"
    )
}

/// `pfctl` arguments for the anchor and the pf enable reference.
pub(crate) mod pfctl {
    use super::PF_ANCHOR;

    /// Replace the anchor's rules with the rules on standard input.
    pub(crate) fn load() -> Vec<String> {
        vec!["-a".into(), PF_ANCHOR.into(), "-f".into(), "-".into()]
    }

    /// Show the loaded rules of the anchor, in pf's own form.
    pub(crate) fn show() -> Vec<String> {
        vec!["-a".into(), PF_ANCHOR.into(), "-s".into(), "rules".into()]
    }

    pub(crate) fn flush() -> Vec<String> {
        vec!["-a".into(), PF_ANCHOR.into(), "-F".into(), "all".into()]
    }

    /// Turn pf on and take a reference token. Other users keep their own.
    pub(crate) fn enable() -> Vec<String> {
        vec!["-E".into()]
    }

    /// Give the reference back. pf stays on while another token holds it.
    pub(crate) fn release(token: u64) -> Vec<String> {
        vec!["-X".into(), token.to_string()]
    }

    pub(crate) fn info() -> Vec<String> {
        vec!["-s".into(), "info".into()]
    }
}

/// Read the token from the output of `pfctl -E`, such as `Token : 1234`.
pub(crate) fn enable_token(output: &str) -> Option<u64> {
    output.lines().find_map(|line| {
        let (key, value) = line.split_once(':')?;
        (key.trim() == "Token").then(|| value.trim().parse().ok())?
    })
}

/// Read whether pf is on from the output of `pfctl -s info`.
pub(crate) fn enabled(info: &str) -> bool {
    info.lines().any(|line| {
        let mut words = line.split_whitespace();
        words.next() == Some("Status:") && words.next() == Some("Enabled")
    })
}

/// The loaded rules match when pf shows the same rule lines as just after
/// the agent's own load. A missing anchor shows no rule lines.
pub(crate) fn same_rules(expected: &str, shown: &str) -> bool {
    let lines = |text: &str| -> Vec<String> {
        text.lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect()
    };
    let expected = lines(expected);
    !expected.is_empty() && expected == lines(shown)
}
