//! Test App credentials, API factory, output capture, and literal store context.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use ciproxy_github::auth::AppJwt;
use ciproxy_github::error::Result;
use ciproxy_github::listener::JobOutput;
use ciproxy_github::persistence::session::ListenerTarget;
use ciproxy_github::runtime::Runtime;
use ciproxy_github::scale_sets::sdk::SdkScaleSetApi;
use ciproxy_github::scale_sets::{ScaleSetApi, ScaleSetFactory};
use ciproxy_store_interface::github::GithubJob;
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::store::Store;
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;
use reqwest::Client;
use scaleset::{Credentials, RetryPolicy};
use secrecy::SecretString;
use uuid::Uuid;

pub(super) fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

pub(super) struct FixedRuntime(pub(super) AtomicI64);
impl Runtime for FixedRuntime {
    fn now(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.0.load(Ordering::SeqCst), 0).unwrap()
    }
    fn uuid(&self) -> Uuid {
        id(1)
    }
}

pub(super) struct FixedJwt;
impl AppJwt for FixedJwt {
    fn token(&self) -> Result<SecretString> {
        Ok("test-app-jwt".into())
    }
}
impl scaleset::JwtProvider for FixedJwt {
    async fn token(&self) -> std::result::Result<String, scaleset::BoxError> {
        Ok("test-app-jwt".to_owned())
    }
}

pub(super) struct Factory(pub(super) String);
#[async_trait]
impl ScaleSetFactory for Factory {
    async fn for_organization(&self, organization: &Organization) -> Result<Arc<dyn ScaleSetApi>> {
        let credentials =
            Credentials::github_app_with_provider(organization.installation_id, FixedJwt).unwrap();
        let http = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let client =
            scaleset::Client::builder(format!("{}/{}", self.0, organization.login), credentials)
                .http_client(http.clone())
                .allow_http()
                .retry_policy(RetryPolicy::none())
                .build()
                .unwrap();
        Ok(Arc::new(SdkScaleSetApi::new(client, http)))
    }
}

#[derive(Default)]
pub(super) struct Output(pub(super) Mutex<Vec<GithubJob>>);
#[async_trait]
impl JobOutput for Output {
    async fn changed(&self, job: GithubJob) -> Result<()> {
        self.0.lock().unwrap().push(job);
        Ok(())
    }
}

pub(super) fn setup(store: &dyn Store) -> ListenerTarget {
    let workspace = Workspace {
        id: id(1),
        name: "customer".to_owned(),
        delete_after: None,
    };
    store.create_workspace(&workspace).unwrap();
    let organization = Organization {
        id: id(5),
        workspace_id: id(1),
        github_id: 17,
        login: "customer".to_owned(),
        installation_id: 19,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    };
    store.install_organization(&organization).unwrap();
    let label = Label {
        id: id(2),
        name: "ciproxy".to_owned(),
        size: VmSize {
            vcpus: 2,
            memory_gb: 8,
            disk_gb: 60,
        },
        allow_machines: true,
        allow_boat: true,
        when_nothing_fits: WhenNothingFits::Wait,
    };
    store.create_label(workspace.id, &label).unwrap();
    store
        .create_scale_set(
            workspace.id,
            &ScaleSet {
                id: id(6),
                organization_id: id(5),
                label_id: id(2),
                github_id: 53,
                runner_group_id: 47,
            },
        )
        .unwrap();
    ListenerTarget {
        workspace_id: id(1),
        organization_id: id(5),
        scale_set_id: id(6),
    }
}
