//! Private Boat launch files and sandbox-scoped callbacks.

use std::future::pending;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::{fs, io::ErrorKind, io::Read};

use async_trait::async_trait;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::secret::Secret;
use reqwest::{Client, redirect::Policy};
use rustix::process::geteuid;
use serde::Deserialize;
use tracing::info;

use crate::boundary::{Channel, CommandFuture};
use crate::error::{Error, ErrorResultExt, Result};
use crate::usage::{ReportFuture, Reporter};

#[derive(Deserialize)]
struct Registration {
    configure: AgentToGuest,
    exit_report_url: Secret,
    exit_report_token: Secret,
}

pub(crate) struct BoatChannel {
    configure: Option<AgentToGuest>,
    callback: Arc<BoatCallback>,
}

struct BoatCallback {
    report_url: Secret,
    token: Secret,
    client: Client,
}

impl BoatCallback {
    async fn post(&self, message: &GuestToAgent) -> Result<()> {
        let response = self
            .client
            .post(self.report_url.expose_secret())
            .bearer_auth(self.token.expose_secret())
            .json(message)
            .send()
            .await
            .context("send scoped Boat report")?;
        response.error_for_status().context("accept Boat report")?;
        Ok(())
    }
}

impl Reporter for BoatCallback {
    fn post_samples(&self, message: GuestToAgent) -> ReportFuture<'_> {
        Box::pin(async move { self.post(&message).await })
    }
}

impl BoatChannel {
    pub(crate) fn from_file(path: &Path) -> Result<Self> {
        let opened = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path);
        let mut file = match opened {
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Err(Error::MissingRegistrationFile);
            }
            other => other.context("open private registration")?,
        };
        let metadata = file.metadata().context("read registration permissions")?;
        if metadata.permissions().mode() & 0o077 != 0 || metadata.uid() != geteuid().as_raw() {
            return Err(Error::InsecureRegistrationFile);
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).context("read registration")?;
        fs::remove_file(path).context("remove consumed registration")?;
        let registration: Registration =
            serde_json::from_slice(&bytes).context("decode private registration")?;
        let client = Client::builder()
            .https_only(true)
            .redirect(Policy::none())
            .timeout(Duration::from_secs(30))
            .build()
            .context("build Boat report client")?;
        Ok(Self {
            configure: Some(registration.configure),
            callback: Arc::new(BoatCallback {
                report_url: registration.exit_report_url,
                token: registration.exit_report_token,
                client,
            }),
        })
    }

    /// Share the callback with the job sampler, without borrowing the channel.
    pub(crate) fn usage_reporter(&self) -> Arc<dyn Reporter> {
        self.callback.clone()
    }
}

#[async_trait]
impl Channel for BoatChannel {
    fn receive(&mut self) -> CommandFuture<'_> {
        Box::pin(async move {
            match self.configure.take() {
                Some(configure) => Ok(Some(configure)),
                None => pending().await,
            }
        })
    }

    async fn send(&mut self, message: GuestToAgent) -> Result<()> {
        if matches!(message, GuestToAgent::RunnerExited { .. }) {
            self.callback.post(&message).await?;
            info!("runner exit report accepted");
        }
        Ok(())
    }
}
