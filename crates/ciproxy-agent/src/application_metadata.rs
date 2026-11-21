//! Select hosted cache metadata only when the machine has a trusted target.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_agent::cache_metadata::client::{RootMetadataClient, TrustedMetadata};
use ciproxy_agent::cache_metadata::connection::connection;
use ciproxy_agent::cache_metadata::error;
use ciproxy_protocol::cache_metadata::{CacheMetadataResponse, CacheMetadataSource};
use ciproxy_protocol::identifiers::JobId;
use reqwest::{Client, redirect};

use crate::error::{ErrorResultExt, Result};
use crate::state::Identity;

/// The root relay's permitted metadata source for this machine invocation.
pub(crate) enum MetadataTarget {
    /// A fresh machine has no credential and no metadata work.
    Disabled,
    /// Retained work gets fallback answers without contacting a remote peer.
    Unavailable,
    /// A known endpoint and credential can resolve hosted facts.
    Hosted(CacheMetadataSource),
}

impl MetadataTarget {
    /// Take the credential and its hosted target from the same identity.
    pub(crate) fn for_machine(identity: Option<&Identity>) -> Self {
        let Some(identity) = identity else {
            return Self::Disabled;
        };
        let source = identity
            .endpoint
            .as_ref()
            .and_then(|endpoint| connection(endpoint.as_str(), identity.credential.clone()));
        match source {
            Some(source) => Self::Hosted(source),
            None => Self::Unavailable,
        }
    }

    /// Keep unavailable replies active when no remote connection is authorized.
    pub(crate) fn client(self) -> Result<Option<Arc<dyn TrustedMetadata>>> {
        match self {
            Self::Disabled => Ok(None),
            Self::Unavailable => Ok(Some(Arc::new(UnavailableMetadata))),
            Self::Hosted(source) => {
                let client = Client::builder()
                    .redirect(redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(5))
                    .timeout(Duration::from_secs(90))
                    .build()
                    .internal()?;
                Ok(Some(Arc::new(RootMetadataClient::new(client, source))))
            }
        }
    }
}

struct UnavailableMetadata;

#[async_trait]
impl TrustedMetadata for UnavailableMetadata {
    async fn lookup(&self, _job_id: JobId) -> error::Result<CacheMetadataResponse> {
        Err(error::Error::Unavailable)
    }
}

#[cfg(test)]
#[path = "_tests_/application_metadata_tests.rs"]
mod application_metadata_tests;
