//! Pre-action assignment socket and in-memory proxy configuration.

use std::os::unix::fs::PermissionsExt;

use ciproxy_protocol::cache::{CacheConnection, DockerDiskAttachment};
use ciproxy_protocol::guest::JobPreparation;
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::jobs::JobFacts;
use ciproxy_protocol::runtime::{CacheProxyConfig, CacheRegistration, RuntimeCredentials};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::process::Command;

use crate::assignment::WorkerReply;
use crate::error::{Error, ErrorResultExt, Result};
use crate::proxy::{self, ProxySession};

#[derive(Clone, Deserialize)]
pub(crate) struct Handoff {
    pub(crate) job_id: JobId,
    pub(crate) runtime: RuntimeCredentials,
}

pub(crate) async fn listen() -> Result<UnixListener> {
    tokio::fs::create_dir_all("/run/ciproxy")
        .await
        .context("create assignment directory")?;
    tokio::fs::set_permissions("/run/ciproxy", std::fs::Permissions::from_mode(0o700))
        .await
        .context("protect assignment directory")?;
    let status = Command::new("chown")
        .args(["runner:runner", "/run/ciproxy"])
        .status()
        .await
        .context("assign handoff directory")?;
    if !status.success() {
        return Err(Error::PoweroffFailed);
    }
    let path = "/run/ciproxy/assignment.sock";
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context("replace assignment socket"),
    }
    let listener = UnixListener::bind(path).context("bind assignment socket")?;
    let status = Command::new("chown")
        .args(["runner:runner", path])
        .status()
        .await
        .context("assign handoff socket")?;
    if !status.success() {
        return Err(Error::PoweroffFailed);
    }
    Ok(listener)
}

pub(crate) async fn read(stream: UnixStream) -> Result<(Handoff, UnixStream)> {
    let mut reader = BufReader::new(stream);
    let mut bytes = Vec::new();
    reader
        .read_until(b'\n', &mut bytes)
        .await
        .context("read trusted assignment")?;
    let mut handoff: Handoff =
        serde_json::from_slice(&bytes).context("decode trusted assignment")?;
    let gateway = Command::new("/usr/local/libexec/ciproxy-container-proxy-url")
        .output()
        .await;
    handoff.runtime.proxy_url = match gateway {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .context("decode trusted Docker gateway")?
            .trim()
            .to_owned(),
        _ => String::new(),
    };
    let data = Command::new("du")
        .args(["--summarize", "--block-size=1", "/var/lib/docker"])
        .output()
        .await;
    handoff.runtime.docker_data_bytes = match data {
        Ok(output) if output.status.success() => String::from_utf8(output.stdout)
            .context("decode preloaded Docker data size")?
            .split_whitespace()
            .next()
            .and_then(|value| value.parse().ok())
            .unwrap_or(0),
        _ => 0,
    };
    Ok((handoff, reader.into_inner()))
}

pub(crate) struct PreparedProxy {
    pub(crate) proxy: Option<Box<dyn ProxySession>>,
    pub(crate) outcome: JobPreparation,
}

pub(crate) async fn prepare(
    handoff: Handoff,
    mut reply: Option<Box<dyn WorkerReply>>,
    facts: JobFacts,
    cache: Option<CacheConnection>,
    docker: Option<DockerDiskAttachment>,
) -> Result<PreparedProxy> {
    let mut outcome = JobPreparation::default();
    let Some(reply) = &mut reply else {
        return Ok(PreparedProxy {
            proxy: None,
            outcome,
        });
    };
    if !reply.begin().await? {
        return Ok(PreparedProxy {
            proxy: None,
            outcome,
        });
    }
    if let Some(disk) = docker {
        let result = Command::new("/usr/local/libexec/ciproxy-docker-attach")
            .arg(disk.free_bytes.to_string())
            .status()
            .await;
        if !matches!(result,Ok(status)if status.success()) {
            tracing::warn!("Docker cache preparation failed; preserve root fallback");
        }
        outcome.docker_attached = matches!(Command::new("mountpoint").args(["--quiet","/var/lib/docker"]).status().await,Ok(status)if status.success());
    }
    let Some(cache) = cache else {
        if reply.ready(None).await.is_err() {
            tracing::warn!("runner already continued with direct GitHub caching");
        }
        return Ok(PreparedProxy {
            proxy: None,
            outcome,
        });
    };
    let runtime = handoff.runtime;
    let proxy_url = runtime.proxy_url.clone();
    let config = CacheProxyConfig {
        machine_url: cache.url,
        machine_token: cache.token,
        registration: CacheRegistration {
            job_id: handoff.job_id,
            facts,
            permissions: runtime.permissions,
            github_token: runtime.github_token,
            cache_url: runtime.cache_url,
            results_url: runtime.results_url,
            proxy_url: proxy_url.clone(),
        },
    };
    match proxy::start(config).await {
        Ok(proxy) => {
            if reply.ready(Some(&proxy_url)).await.is_ok() {
                outcome.cache_ready = true;
                Ok(PreparedProxy {
                    proxy: Some(proxy),
                    outcome,
                })
            } else {
                tracing::warn!("runner readiness reply failed; preserve GitHub cache accounting");
                Ok(PreparedProxy {
                    proxy: None,
                    outcome,
                })
            }
        }
        Err(error) => {
            tracing::warn!(%error,"VM cache proxy unavailable; preserve GitHub fallback");
            let _ = reply.ready(None).await;
            Ok(PreparedProxy {
                proxy: None,
                outcome,
            })
        }
    }
}
