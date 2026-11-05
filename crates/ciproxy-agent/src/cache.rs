//! Optional local cache control. An absent cache keeps jobs on GitHub caching.

use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::Duration;

use async_trait::async_trait;
use ciproxy_protocol::cache::{CacheConnection, MachineCacheReport};
use ciproxy_protocol::identifiers::{OrganizationId, RunnerId};
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::secret::Secret;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::boundaries::Cache;
use crate::error::{Error, ErrorResultExt, Result};
use crate::network::host_address;
use crate::reporting::empty_cache;
use crate::state::VmRecord;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Request<'a> {
    Budget {
        capacity_bytes: u64,
    },
    Report,
    Clear,
    Configure {
        runner_id: &'a RunnerId,
        organization_id: &'a OrganizationId,
        address: String,
    },
    Facts {
        runner_id: &'a RunnerId,
        facts: &'a JobFacts,
    },
    Release {
        runner_id: &'a RunnerId,
    },
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Response {
    Report { report: MachineCacheReport },
    Configured { token: Secret },
    Applied,
}

pub(crate) struct UnixCache {
    pub(crate) socket: PathBuf,
}

#[async_trait]
impl Cache for UnixCache {
    async fn budget(&self, capacity_bytes: u64) -> Result<()> {
        self.applied(Request::Budget { capacity_bytes }).await
    }

    async fn report(&self) -> Result<MachineCacheReport> {
        match self.request(Request::Report).await? {
            Some(Response::Report { report }) => Ok(report),
            None => Ok(empty_cache()),
            _ => Err(Error::GuestMismatch),
        }
    }

    async fn clear(&self) -> Result<()> {
        match self.request(Request::Clear).await? {
            Some(Response::Applied) => Ok(()),
            _ => Err(Error::CacheUnavailable),
        }
    }

    async fn configure(&self, vm: &VmRecord) -> Result<Option<CacheConnection>> {
        let address = format!("http://{}:7887", host_address(vm.network_index));
        let response = self
            .request(Request::Configure {
                runner_id: &vm.report.runner_id,
                organization_id: &vm.report.organization_id,
                address: address.clone(),
            })
            .await?;
        match response {
            Some(Response::Configured { token }) => Ok(Some(CacheConnection {
                url: Secret::new(address),
                token,
            })),
            None => Ok(None),
            _ => Err(Error::GuestMismatch),
        }
    }

    async fn facts(&self, runner: &RunnerId, facts: &JobFacts) -> Result<()> {
        self.applied(Request::Facts {
            runner_id: runner,
            facts,
        })
        .await
    }

    async fn release(&self, runner: &RunnerId) -> Result<()> {
        self.applied(Request::Release { runner_id: runner }).await
    }
}

impl UnixCache {
    async fn applied(&self, request: Request<'_>) -> Result<()> {
        match self.request(request).await? {
            Some(Response::Applied) | None => Ok(()),
            _ => Err(Error::GuestMismatch),
        }
    }

    async fn request(&self, request: Request<'_>) -> Result<Option<Response>> {
        match std::fs::metadata(&self.socket) {
            Ok(metadata) if metadata.uid() == 0 && metadata.mode() & 0o077 == 0 => {}
            Ok(_) => return Err(Error::IdentityMismatch),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("inspect cache control socket"),
        }
        timeout(Duration::from_secs(2), async {
            let stream = UnixStream::connect(&self.socket)
                .await
                .context("connect cache control")?;
            let mut stream = BufReader::new(stream);
            let mut bytes = serde_json::to_vec(&request).context("encode cache control")?;
            bytes.push(b'\n');
            stream
                .write_all(&bytes)
                .await
                .context("send cache control")?;
            let mut line = Vec::new();
            stream
                .read_until(b'\n', &mut line)
                .await
                .context("read cache control")?;
            Ok(Some(
                serde_json::from_slice(&line).context("decode cache control")?,
            ))
        })
        .await
        .context("cache control timeout")?
    }
}
