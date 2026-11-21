//! Owned ingress and service-UID output rules protect every cache start.

use std::sync::Arc;

use async_trait::async_trait;

use crate::cache_unit::FIREWALL_PATH;
use crate::error::Result;
use crate::process::{Commands, arguments};

const ACCOUNT: &str = "u ciproxy-cache - \"ciproxy machine cache\" /nonexistent\n";

/// Only reply-direction TCP bypasses destination blocks for accepted clients.
pub(crate) const RULES: &str = r#"add table inet ciproxy_cache
add chain inet ciproxy_cache input { type filter hook input priority -20; policy accept; }
flush chain inet ciproxy_cache input
add rule inet ciproxy_cache input tcp dport 7887 iifname != "lo" iifname != "cpx*" counter drop;
add chain inet ciproxy_cache output { type filter hook output priority -20; policy accept; }
add chain inet ciproxy_cache service
add set inet ciproxy_cache dns4 { type ipv4_addr; }
add set inet ciproxy_cache dns6 { type ipv6_addr; }
flush chain inet ciproxy_cache output
flush chain inet ciproxy_cache service
flush set inet ciproxy_cache dns4
flush set inet ciproxy_cache dns6
add rule inet ciproxy_cache output meta skuid "ciproxy-cache" jump service
add rule inet ciproxy_cache service ct direction reply ct state established tcp sport { 7887, 8229 } return
add rule inet ciproxy_cache service ip daddr @dns4 udp dport 53 return
add rule inet ciproxy_cache service ip daddr @dns4 tcp dport 53 return
add rule inet ciproxy_cache service ip6 daddr @dns6 udp dport 53 return
add rule inet ciproxy_cache service ip6 daddr @dns6 tcp dport 53 return
add rule inet ciproxy_cache service meta nfproto ipv6 counter drop
add rule inet ciproxy_cache service fib daddr type { local, broadcast, multicast } counter drop
add rule inet ciproxy_cache service ip daddr { 0.0.0.0/8, 10.0.0.0/8, 100.64.0.0/10, 127.0.0.0/8, 169.254.0.0/16, 172.16.0.0/12, 192.0.0.0/24, 192.0.2.0/24, 192.168.0.0/16, 198.18.0.0/15, 198.51.100.0/24, 203.0.113.0/24, 224.0.0.0/4, 240.0.0.0/4, 168.63.129.16/32 } counter drop
"#;

#[async_trait]
pub(crate) trait CacheStart: Send + Sync {
    async fn repair(&self) -> Result<()>;
    async fn start(&self, restart: bool) -> Result<()>;
}

pub(crate) struct GuardedCacheStart {
    pub(crate) commands: Arc<dyn Commands>,
}

#[async_trait]
impl CacheStart for GuardedCacheStart {
    async fn repair(&self) -> Result<()> {
        self.commands
            .run("systemd-sysusers", &arguments(&["-"]), ACCOUNT.as_bytes())
            .await?;
        self.commands
            .run("/bin/sh", &arguments(&[FIREWALL_PATH]), &[])
            .await?;
        Ok(())
    }

    async fn start(&self, restart: bool) -> Result<()> {
        self.repair().await?;
        if restart {
            self.commands
                .run(
                    "systemctl",
                    &arguments(&["try-restart", "ciproxy-cache.service"]),
                    &[],
                )
                .await?;
        }
        self.commands
            .run(
                "systemctl",
                &arguments(&["enable", "--now", "ciproxy-cache.service"]),
                &[],
            )
            .await?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "_tests_/cache_network_tests.rs"]
mod cache_network_tests;
