//! Bounded authenticated Boat reads that never follow redirects.

use std::time::Duration;

use ciproxy_protocol::secret::Secret;
use reqwest::{Certificate, Client, redirect::Policy};
use serde::de::DeserializeOwned;
use url::Url;

use crate::error::{Error, Result, ResultExt};

pub(super) struct BoatReads {
    base: String,
    client: Client,
    key: Secret,
    pub(super) wallet: String,
}

impl BoatReads {
    pub(super) fn new(
        base: String,
        key: Secret,
        wallet: String,
        ca: Option<&[u8]>,
    ) -> Result<Self> {
        let mut builder = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .timeout(Duration::from_secs(30));
        if let Some(ca) = ca {
            builder = builder.add_root_certificate(Certificate::from_pem(ca).internal()?);
        }
        Ok(Self {
            base,
            client: builder.build().internal()?,
            key,
            wallet,
        })
    }

    pub(super) async fn get<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<Option<T>> {
        let mut url = Url::parse(&format!("{}{path}", self.base)).internal()?;
        for (key, value) in query {
            url.query_pairs_mut().append_pair(key, value);
        }
        let mut response = self
            .client
            .get(url)
            .bearer_auth(self.key.expose_secret())
            .header("X-Boat-Org", &self.wallet)
            .send()
            .await
            .internal()?;
        match response.status().as_u16() {
            200 => {}
            401 | 403 => return Err(Error::RejectedKey),
            404 => return Ok(None),
            _ => return Err(Error::ProviderUnavailable),
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.internal()? {
            if chunk.len() > (1024 * 1024usize).saturating_sub(bytes.len()) {
                return Err(Error::InvalidProviderData);
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(Some(serde_json::from_slice(&bytes).internal()?))
    }
}

pub(super) fn valid_sandbox(id: &str) -> bool {
    id.len() == 11
        && id.starts_with("bx_")
        && id.as_bytes()[3..]
            .iter()
            .all(|b| b"23456789abcdefghjkmnpqrstuvwxyz".contains(b))
}
