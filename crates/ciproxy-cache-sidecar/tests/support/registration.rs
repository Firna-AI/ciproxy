//! Per-job signed assignments delivered through the authenticated admin API.

use std::sync::Arc;

use ciproxy_cache_sidecar::proxy::{ProxyConfig, VmProxy};
use ciproxy_cache_sidecar::service::{Service, job_router};
use ciproxy_cache_sidecar::session::{Permissions, Registration, SessionCredential};
use ciproxy_protocol::identifiers::JobId;
use ciproxy_protocol::secret::Secret;
use tokio::net::TcpListener;

use super::Fixture;
use super::job::Job;
use super::tokens::JobSpec;

impl Fixture {
    pub async fn job(&self, id: &str, facts: JobSpec) -> Job {
        self.job_with(
            id,
            facts,
            Permissions {
                read: true,
                write: true,
            },
            false,
        )
        .await
    }

    pub async fn job_with(
        &self,
        id: &str,
        facts: JobSpec,
        permissions: Permissions,
        machine_down: bool,
    ) -> Job {
        let machine = if machine_down {
            "http://127.0.0.1:1"
        } else {
            &self.machine
        };
        self.job_via(id, facts, permissions, machine).await
    }

    pub async fn job_via(
        &self,
        id: &str,
        mut facts: JobSpec,
        permissions: Permissions,
        machine_url: &str,
    ) -> Job {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        facts.assignment.job_id = Some(JobId(id.into()));
        let github_token = facts.runtime_token();
        let registration = Registration {
            job_id: JobId(id.into()),
            facts: facts.facts,
            permissions,
            github_token: github_token.clone(),
            cache_url: Secret::new(format!("{}/legacy-prefix/", self.github.origin)),
            results_url: Secret::new(&self.github.origin),
            proxy_url: origin.clone(),
        };
        let credential: SessionCredential = self
            .client
            .post(format!("{}/sessions", self.admin))
            .bearer_auth("admin-secret")
            .json(&registration)
            .send()
            .await
            .unwrap()
            .error_for_status()
            .unwrap()
            .json()
            .await
            .unwrap();
        let proxy: Arc<dyn Service> = Arc::new(VmProxy::new(
            ProxyConfig {
                machine_url: Secret::new(machine_url),
                machine_token: credential.token.clone(),
                registration,
            },
            self.network.clone(),
        ));
        let task = tokio::spawn(async move {
            axum::serve(listener, job_router(proxy)).await.unwrap();
        });
        Job {
            origin,
            client: self.client.clone(),
            token: credential.token,
            github_token,
            scope: credential.scope,
            task,
        }
    }
}
