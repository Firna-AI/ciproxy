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

use super::failure::{Operation, matches, missing};
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
    async fn ensure(
        &self,
        label: &str,
        runner_group_id: i64,
        saved_id: Option<i64>,
    ) -> Result<i64> {
        let group = RunnerGroupId(runner_group_id);
        if let Some(id) = saved_id {
            match self.client.get_runner_scale_set_by_id(ScaleSetId(id)).await {
                Ok(existing) => {
                    if existing.runner_group_id != group {
                        return Err(Error::ScaleSetGroupChanged {
                            scale_set_id: id,
                            expected_group: runner_group_id,
                            actual_group: existing.runner_group_id.get(),
                        });
                    }
                    return Ok(existing.id.get());
                }
                Err(error) if missing(&error, Operation::ScaleSet(id)) => {}
                Err(error) => return Err(error).internal(),
            }
        }
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
        let session = match self
            .client
            .create_message_session(ScaleSetId(scale_set_id), owner)
            .await
        {
            Ok(session) => session,
            Err(error) if missing(&error, Operation::Open(scale_set_id)) => {
                return Err(Error::ScaleSetMissing { scale_set_id });
            }
            Err(error) => return Err(error).internal(),
        };
        Ok(Box::new(SdkQueue {
            session,
            http: self.http.clone(),
        }))
    }

    async fn register(&self, scale_set_id: i64, name: &str) -> Result<JitRegistration> {
        let jit = match self
            .client
            .generate_jit_config(ScaleSetId(scale_set_id), &JitRunnerSetting::new(name))
            .await
        {
            Ok(jit) => jit,
            Err(error) if missing(&error, Operation::Jit(scale_set_id)) => {
                return Err(Error::ScaleSetMissing { scale_set_id });
            }
            Err(error) => return Err(error).internal(),
        };
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
            Err(error) if missing(&error, Operation::Runner(runner_id)) => Ok(()),
            Err(error)
                if error.is_job_still_running()
                    && matches(&error, Operation::Runner(runner_id)) =>
            {
                Err(Error::RunnerBusy)
            }
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
            Err(error) if missing(&error, Operation::ScaleSet(scale_set_id)) => Ok(()),
            Err(error) => Err(error).internal(),
        }
    }
}
