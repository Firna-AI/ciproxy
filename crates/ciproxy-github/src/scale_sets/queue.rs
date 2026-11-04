//! Lossless typed polling of SDK sessions, keeping unreadable work unacknowledged.

use std::time::Duration;

use async_trait::async_trait;
use reqwest::{Client, StatusCode, Url};
use scaleset::{JobMessage, MessageId, MessageSession, ScaleSetMessage, ScaleSetStatistics};
use secrecy::ExposeSecret;
use serde::Deserialize;

use crate::error::{Error, Result, ResultExt};

use super::MessageQueue;

pub(super) struct SdkQueue {
    pub(super) session: MessageSession,
    pub(super) http: Client,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    message_id: MessageId,
    message_type: String,
    body: String,
    statistics: Option<ScaleSetStatistics>,
}

#[async_trait]
impl MessageQueue for SdkQueue {
    async fn poll(
        &self,
        last_message_id: Option<MessageId>,
        capacity: u32,
    ) -> Result<Option<ScaleSetMessage>> {
        for attempt in 0..2 {
            let info = self.session.info();
            let mut url = Url::parse(&info.message_queue_url).internal()?;
            if url.scheme() != "https"
                && self.session.client().config_url().url().scheme() != "http"
            {
                return Err(scaleset::Error::InsecureUrl {
                    url: url.origin().ascii_serialization(),
                })
                .internal();
            }
            if let Some(id) = last_message_id {
                url.query_pairs_mut()
                    .append_pair("lastMessageId", &id.to_string());
            }
            let response = self
                .http
                .get(url)
                .header("Accept", "application/json; api-version=6.0-preview")
                .header("X-ScaleSetMaxCapacity", capacity)
                .header(
                    "User-Agent",
                    concat!("ciproxy-github/", env!("CARGO_PKG_VERSION")),
                )
                .bearer_auth(info.message_queue_access_token.expose_secret())
                .timeout(Duration::from_secs(65))
                .send()
                .await
                .internal()?;
            if response.status() == StatusCode::ACCEPTED {
                return Ok(None);
            }
            if response.status() == StatusCode::UNAUTHORIZED && attempt == 0 {
                self.session
                    .get_message(last_message_id, capacity)
                    .await
                    .internal()?;
                continue;
            }
            if response.status() != StatusCode::OK {
                return Err(Error::AccessUnavailable(response.status().as_u16()));
            }
            let raw: Envelope = response.json().await.internal()?;
            let jobs: Vec<JobMessage> = if raw.message_type != "RunnerScaleSetJobMessages" {
                tracing::warn!("GitHub sent an unknown scale-set envelope type");
                Vec::new()
            } else if raw.body.is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&raw.body).internal()?
            };
            let mut message = ScaleSetMessage::default();
            message.id = raw.message_id;
            message.statistics = raw.statistics;
            message.jobs = jobs;
            return Ok(Some(message));
        }
        Err(Error::AccessUnavailable(401))
    }

    async fn acquire(&self, requests: &[scaleset::JobRequestId]) -> Result<()> {
        self.session.acquire_jobs(requests).await.internal()?;
        Ok(())
    }
    async fn acknowledge(&self, message_id: MessageId) -> Result<()> {
        self.session.delete_message(message_id).await.internal()
    }
    async fn close(&self) -> Result<()> {
        self.session.close().await.internal()
    }
}
