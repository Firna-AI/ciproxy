//! Authenticated webhook dispatch for the hosted /hooks/github route.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;

use crate::app::webhook::WebhookVerifier;
use crate::app::{GitHubApp, InstallationOutcome};
use crate::error::{Result, ResultExt};

/// The result returned to the hosted route after a verified delivery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WebhookOutcome {
    /// An App installation lifecycle event completed.
    Installation(InstallationOutcome),
    /// Installation repository visibility or repository metadata refreshed.
    RepositoriesRefreshed,
    /// GitHub sent an event that this integration does not consume.
    Ignored,
}

/// The hosted route's side-effecting webhook boundary.
#[unimock::unimock(api = WebhookHandlerMock)]
#[async_trait]
pub trait WebhookHandler: Send + Sync {
    /// Verify the raw body before parsing or dispatching its event.
    async fn handle(
        &self,
        event_name: &str,
        signature: &str,
        body: &[u8],
    ) -> Result<WebhookOutcome>;
}

/// The authenticated dispatcher constructed by the hosted composition root.
pub struct AppWebhooks {
    /// The signature verifier holding the configured webhook secret.
    pub verifier: Arc<dyn WebhookVerifier>,
    /// The App service consumed through its public boundary.
    pub app: Arc<dyn GitHubApp>,
}

#[derive(Deserialize)]
struct RepositoryEvent {
    installation: Option<InstallationIdentity>,
}

#[derive(Deserialize)]
struct InstallationIdentity {
    id: i64,
}

#[async_trait]
impl WebhookHandler for AppWebhooks {
    async fn handle(
        &self,
        event_name: &str,
        signature: &str,
        body: &[u8],
    ) -> Result<WebhookOutcome> {
        self.verifier.verify(signature, body).await?;
        match event_name {
            "installation" => {
                let event = serde_json::from_slice(body).internal()?;
                Ok(WebhookOutcome::Installation(
                    self.app.installation(event).await?,
                ))
            }
            "installation_repositories" | "repository" => {
                let event: RepositoryEvent = serde_json::from_slice(body).internal()?;
                if let Some(installation) = event.installation {
                    self.app.refresh_repositories(installation.id).await?;
                    Ok(WebhookOutcome::RepositoriesRefreshed)
                } else {
                    Ok(WebhookOutcome::Ignored)
                }
            }
            _ => Ok(WebhookOutcome::Ignored),
        }
    }
}
