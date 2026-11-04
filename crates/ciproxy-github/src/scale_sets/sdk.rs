//! scaleset 0.2 adapters, with caller-owned session acknowledgement.

use std::sync::Arc;

use async_trait::async_trait;
use ciproxy_store_interface::organizations::Organization;
use reqwest::Client;
use scaleset::{
    Credentials, JitRunnerSetting, RunnerGroupId, RunnerId, RunnerScaleSetSpec, ScaleSetId,
};

use crate::auth::{AppJwt, ScaleSetJwt};
use crate::error::{Error, Result, ResultExt};

use super::queue::SdkQueue;
use super::{JitRegistration, MessageQueue, ScaleSetApi, ScaleSetFactory};

/// Production API factory using the App signer of the control plane.
pub struct AppScaleSetFactory {
    signer: Arc<dyn AppJwt>,
    http: Client,
}

impl AppScaleSetFactory {
    /// Construct the factory with a non-redirecting HTTP client.
    pub fn new(signer: Arc<dyn AppJwt>, http: Client) -> Self {
        Self { signer, http }
    }
}

#[async_trait]
impl ScaleSetFactory for AppScaleSetFactory {
    async fn for_organization(&self, organization: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        let credentials = Credentials::github_app_with_provider(
            organization.installation_id,
            ScaleSetJwt {
                signer: Arc::clone(&self.signer),
            },
        )
        .internal()?;
        let client = scaleset::Client::builder(
            format!("https://github.com/{}", organization.login),
            credentials,
        )
        .http_client(self.http.clone())
        .build()
        .internal()?;
        Ok(Arc::new(SdkScaleSetApi::new(client, self.http.clone())))
    }
}

/// The real SDK adapter. Tests can supply an SDK client for a fake HTTP server.
pub struct SdkScaleSetApi {
    client: scaleset::Client,
    http: Client,
}

impl SdkScaleSetApi {
    /// Bind a configured SDK client to the narrow integration boundary.
    pub fn new(client: scaleset::Client, http: Client) -> Self {
        Self { client, http }
    }
}

#[async_trait]
impl ScaleSetApi for SdkScaleSetApi {
    async fn ensure(&self, label: &str, runner_group_id: i64) -> Result<i64> {
        let group = RunnerGroupId(runner_group_id);
        if let Some(existing) = self
            .client
            .get_runner_scale_set_by_name(group, label)
            .await
            .internal()?
        {
            return Ok(existing.id.get());
        }
        let spec = RunnerScaleSetSpec::new(label)
            .runner_group_id(group)
            .disable_update();
        let created = self
            .client
            .create_runner_scale_set(&spec)
            .await
            .internal()?;
        Ok(created.id.get())
    }

    async fn open(&self, scale_set_id: i64, owner: &str) -> Result<Box<dyn MessageQueue>> {
        let session = self
            .client
            .create_message_session(ScaleSetId(scale_set_id), owner)
            .await
            .internal()?;
        Ok(Box::new(SdkQueue {
            session,
            http: self.http.clone(),
        }))
    }

    async fn register(&self, scale_set_id: i64, name: &str) -> Result<JitRegistration> {
        let jit = self
            .client
            .generate_jit_config(ScaleSetId(scale_set_id), &JitRunnerSetting::new(name))
            .await
            .internal()?;
        let Some(runner) = jit.runner else {
            return Err(Error::OrganizationInactive);
        };
        Ok(JitRegistration {
            runner_id: runner.id.get(),
            configuration: jit.encoded_jit_config,
        })
    }

    async fn remove_runner(&self, runner_id: i64) -> Result<()> {
        match self.client.remove_runner(RunnerId(runner_id)).await {
            Ok(()) => Ok(()),
            Err(error) if error.is_not_found() || error.is_runner_not_found() => Ok(()),
            Err(error) if error.is_job_still_running() => Err(Error::RunnerBusy),
            Err(error) => Err(error).internal(),
        }
    }

    async fn remove_scale_set(&self, scale_set_id: i64) -> Result<()> {
        match self
            .client
            .delete_runner_scale_set(ScaleSetId(scale_set_id))
            .await
        {
            Ok(()) => Ok(()),
            Err(error) if error.is_not_found() => Ok(()),
            Err(error) => Err(error).internal(),
        }
    }
}
