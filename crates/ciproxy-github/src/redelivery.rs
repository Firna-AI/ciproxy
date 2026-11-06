//! Recover failed App webhook deliveries, including failures before persistence.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{Client, Url};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::auth::AppJwt;
use crate::error::{Error, Result, ResultExt};

#[derive(Debug, thiserror::Error)]
#[error("[github/redelivery] repeated delivery-history cursor")]
struct PaginationCycle;

/// One GitHub delivery attempt, without request bodies or signature bytes.
#[derive(Clone, Debug, Deserialize)]
pub struct Delivery {
    /// Numeric attempt used by the redelivery endpoint.
    pub id: u64,
    /// Stable identity shared by every retry of the same event.
    pub guid: String,
    /// Trusted attempt time supplied by GitHub.
    pub delivered_at: DateTime<Utc>,
    /// HTTP status, absent when the remote listener could not be contacted.
    pub status_code: Option<u16>,
    /// Event kind used to restrict recovery to this integration's handlers.
    pub event: String,
}

/// One cursor page from GitHub's delivery history.
pub struct Page {
    /// Metadata of attempted deliveries.
    pub deliveries: Vec<Delivery>,
    /// Next cursor from GitHub's Link header.
    pub next: Option<String>,
}

/// App-JWT delivery operations. No installation or extra App permission is used.
#[cfg_attr(test, unimock::unimock(api = DeliveryApiMock))]
#[async_trait]
pub trait DeliveryApi: Send + Sync {
    /// Read one page of retained App delivery metadata.
    async fn list(&self, cursor: Option<String>) -> Result<Page>;
    /// Ask GitHub to redeliver this failed attempt to the configured hook.
    async fn redeliver(&self, id: u64) -> Result<()>;
}

/// Continuous runtime recovery boundary.
#[async_trait]
pub trait WebhookRecovery: Send + Sync {
    /// Retry only the latest failed attempt of each relevant event.
    async fn retry_failed(&self) -> Result<()>;
}

/// Retained GitHub history supplies recovery after a control-plane restart.
pub struct RecoverDeliveries {
    /// Authenticated remote history and redelivery operations.
    pub api: Arc<dyn DeliveryApi>,
}

#[async_trait]
impl WebhookRecovery for RecoverDeliveries {
    async fn retry_failed(&self) -> Result<()> {
        let mut latest = HashMap::<String, Delivery>::new();
        let mut cursor = None;
        let mut cursors = HashSet::new();
        loop {
            let page = self.api.list(cursor).await?;
            for delivery in page.deliveries {
                if !matches!(
                    delivery.event.as_str(),
                    "installation" | "installation_repositories" | "repository"
                ) {
                    continue;
                }
                let old = latest
                    .entry(delivery.guid.clone())
                    .or_insert_with(|| delivery.clone());
                if (delivery.delivered_at, delivery.id) > (old.delivered_at, old.id) {
                    *old = delivery;
                }
            }
            cursor = page.next;
            if let Some(next) = &cursor {
                if !cursors.insert(next.clone()) {
                    return Err(PaginationCycle).internal();
                }
            } else {
                break;
            }
        }
        let mut failed: Vec<_> = latest
            .values()
            .filter(|delivery| {
                !delivery
                    .status_code
                    .is_some_and(|status| (200..400).contains(&status))
            })
            .collect();
        failed.sort_by_key(|delivery| (delivery.delivered_at, delivery.id));
        for delivery in failed {
            if let Err(error) = self.api.redeliver(delivery.id).await {
                tracing::warn!(id=delivery.id,%error,"failed webhook redelivery will retry from GitHub history");
            }
        }
        Ok(())
    }
}

/// Production delivery API with the same App signer and bounded HTTP client.
pub struct HttpDeliveries {
    /// Non-redirecting hosted HTTP client.
    pub http: Client,
    /// GitHub REST origin, or an isolated integration fixture.
    pub api: Url,
    /// App JWT signer retained only in the hosted process.
    pub signer: Arc<dyn AppJwt>,
}

impl HttpDeliveries {
    fn request(&self, method: reqwest::Method, url: Url) -> Result<reqwest::RequestBuilder> {
        Ok(self
            .http
            .request(method, url)
            .bearer_auth(self.signer.token()?.expose_secret())
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "ciproxy-control-plane")
            .header("X-GitHub-Api-Version", "2026-03-10"))
    }
}

#[async_trait]
impl DeliveryApi for HttpDeliveries {
    async fn list(&self, cursor: Option<String>) -> Result<Page> {
        let mut url = self.api.join("app/hook/deliveries").internal()?;
        url.query_pairs_mut().append_pair("per_page", "100");
        if let Some(cursor) = cursor {
            url.query_pairs_mut().append_pair("cursor", &cursor);
        }
        let response = self
            .request(reqwest::Method::GET, url)?
            .send()
            .await
            .internal()?;
        if !response.status().is_success() {
            return Err(Error::AccessUnavailable(response.status().as_u16()));
        }
        let next = response
            .headers()
            .get("link")
            .and_then(|header| header.to_str().ok())
            .and_then(|links| links.split(',').find(|link| link.contains("rel=\"next\"")))
            .and_then(|link| link.trim().strip_prefix('<'))
            .and_then(|link| link.split_once('>').map(|(url, _)| url))
            .and_then(|url| Url::parse(url).ok())
            .and_then(|url| {
                url.query_pairs()
                    .find(|(key, _)| key == "cursor")
                    .map(|(_, value)| value.into_owned())
            });
        Ok(Page {
            deliveries: response.json().await.internal()?,
            next,
        })
    }

    async fn redeliver(&self, id: u64) -> Result<()> {
        let response = self
            .request(
                reqwest::Method::POST,
                self.api
                    .join(&format!("app/hook/deliveries/{id}/attempts"))
                    .internal()?,
            )?
            .send()
            .await
            .internal()?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::AccessUnavailable(response.status().as_u16()))
        }
    }
}

#[cfg(test)]
#[path = "_tests_/redelivery_tests.rs"]
mod redelivery_tests;
