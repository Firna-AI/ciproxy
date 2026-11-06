//! One owned cache ingress chain protects every VM without a physical listener.

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::process::{Commands, arguments};

pub(crate) const RULES: &str = r#"add table inet ciproxy_cache
add chain inet ciproxy_cache input { type filter hook input priority -20; policy accept; }
flush chain inet ciproxy_cache input
add rule inet ciproxy_cache input tcp dport 7887 iifname != "lo" iifname != "cpx*" drop;
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
            .run("nft", &arguments(&["-f", "-"]), RULES.as_bytes())
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
