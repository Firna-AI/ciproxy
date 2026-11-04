//! nftables policy. IPv6 and all non-public IPv4 ranges stay outside the guest.

use crate::network::{host_address, network_name, uplink_address};

const NON_PUBLIC: &str = "0.0.0.0/8, 10.0.0.0/8, 100.64.0.0/10, 127.0.0.0/8, 169.254.0.0/16, 172.16.0.0/12, 192.0.0.0/24, 192.0.2.0/24, 192.168.0.0/16, 198.18.0.0/15, 198.51.100.0/24, 203.0.113.0/24, 224.0.0.0/4, 240.0.0.0/4, 168.63.129.16/32";

pub(crate) fn namespace_rules(index: u16) -> String {
    let host = host_address(index);
    format!(
        r#"table inet ciproxy {{
 chain input {{ type filter hook input priority 0; policy drop; }}
 chain forward {{ type filter hook forward priority 0; policy drop;
  meta nfproto ipv6 drop
  iifname "uplink" oifname "tap0" ct state established,related accept
  iifname "tap0" ip saddr != 172.30.0.2 drop
  iifname "tap0" oifname "uplink" ip daddr {host} tcp dport 7887 accept
  iifname "tap0" ip daddr {{ {NON_PUBLIC} }} drop
  iifname "tap0" oifname "uplink" accept
 }}
 chain nat {{ type nat hook postrouting priority srcnat; policy accept;
  oifname "uplink" ip saddr 172.30.0.2 masquerade
 }}
}}
"#
    )
}

pub(crate) fn root_rules(index: u16) -> String {
    let id = network_name(index);
    let host = host_address(index);
    let uplink = uplink_address(index);
    format!(
        r#"table inet {id} {{
 chain input {{ type filter hook input priority -10; policy accept;
  iifname "{id}" ip daddr {host} tcp dport 7887 accept
  iifname "{id}" drop
 }}
 chain forward {{ type filter hook forward priority -10; policy accept;
  iifname "{id}" meta nfproto ipv6 drop
  iifname "{id}" ip saddr != {uplink} drop
  iifname "{id}" ip daddr {{ {NON_PUBLIC} }} drop
  oifname "{id}" ct state != established,related drop
 }}
 chain nat {{ type nat hook postrouting priority srcnat; policy accept;
  ip saddr {uplink} oifname != "{id}" masquerade
 }}
}}
"#
    )
}
