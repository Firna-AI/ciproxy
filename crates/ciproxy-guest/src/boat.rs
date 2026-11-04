//! Private Boat launch files and sandbox-scoped runner-exit callbacks.

use std::future::pending;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::time::Duration;
use std::{fs, io::Read};

use async_trait::async_trait;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::secret::Secret;
use reqwest::{Client, redirect::Policy};
use rustix::process::geteuid;
use serde::Deserialize;
use tracing::info;

use crate::boundary::{Channel, CommandFuture};
use crate::error::{Error, ErrorResultExt, Result};

#[derive(Deserialize)]
struct Registration {
    configure: AgentToGuest,
    exit_report_url: Secret,
    exit_report_token: Secret,
}

pub(crate) struct BoatChannel {
    configure: Option<AgentToGuest>,
    report_url: Secret,
    token: Secret,
    client: Client,
}

impl BoatChannel {
    pub(crate) fn from_file(path: &Path) -> Result<Self> {
        let mut file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW)
            .open(path)
            .context("open private registration")?;
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
            .context("build exit-report client")?;
        Ok(Self {
            configure: Some(registration.configure),
            report_url: registration.exit_report_url,
            token: registration.exit_report_token,
            client,
        })
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
            let response = self
                .client
                .post(self.report_url.expose_secret())
                .bearer_auth(self.token.expose_secret())
                .json(&message)
                .send()
                .await
                .context("send scoped exit report")?;
            response.error_for_status().context("accept exit report")?;
            info!("runner exit report accepted");
        }
        Ok(())
    }
}
