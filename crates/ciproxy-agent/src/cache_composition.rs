//! Compose and own the root metadata task separately from the cache service.

use std::sync::Arc;
use std::time::Duration;

use ciproxy_agent::cache_metadata::administration::HttpMetadataAdministration;
use ciproxy_agent::cache_metadata::client::RootMetadataClient;
use ciproxy_agent::cache_metadata::relay::{MetadataRelay, RootMetadataRelay, TokioRelayDelay};
use ciproxy_protocol::cache_metadata::CacheMetadataSource;
use reqwest::{Client, redirect};
use tokio::task::JoinHandle;

use crate::boundaries::Cache;
use crate::cache::HttpCache;
use crate::cache_admin::{HttpAdministration, OwnedCacheFiles};
use crate::error::{ErrorResultExt, Result};
use crate::process::Commands;

/// Cache services owned by one root agent invocation.
pub(crate) struct CacheServices {
    /// Cache lifecycle operations given to the controller.
    pub(crate) cache: Arc<dyn Cache>,
    task: Option<JoinHandle<()>>,
}

/// Build cache administration and start the root-only relay when identity exists.
pub(crate) fn services(
    commands: Arc<dyn Commands>,
    source: Option<CacheMetadataSource>,
) -> Result<CacheServices> {
    let token_file = "/var/lib/ciproxy/cache-admin-token".into();
    let cache: Arc<dyn Cache> = Arc::new(HttpCache::with_boundaries(
        Arc::new(HttpAdministration::new(token_file, source.is_some())?),
        Arc::new(OwnedCacheFiles {
            directory: "/var/lib/ciproxy/area/cache".into(),
            commands: commands.clone(),
        }),
        commands,
    ));
    let task = if let Some(source) = source {
        let client = Client::builder()
            .redirect(redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(90))
            .build()
            .internal()?;
        let relay: Arc<dyn MetadataRelay> = Arc::new(RootMetadataRelay {
            administration: Arc::new(HttpMetadataAdministration::new(
                client.clone(),
                "http://127.0.0.1:8229".into(),
                "/var/lib/ciproxy/cache-admin-token".into(),
            )),
            hosted: Arc::new(RootMetadataClient::new(client, source)),
            delay: Arc::new(TokioRelayDelay),
        });
        Some(tokio::spawn(async move {
            relay.run().await;
        }))
    } else {
        None
    };
    Ok(CacheServices { cache, task })
}

impl Drop for CacheServices {
    fn drop(&mut self) {
        if let Some(task) = &self.task {
            task.abort();
        }
    }
}
