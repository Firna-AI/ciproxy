//! Construct hosted adapters without exposing secrets to agents or API clients.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock, Weak};
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use ciproxy_gateway::boundary::{AgentEvents, Authorization, SystemClock, SystemEntropy};
use ciproxy_gateway::connections::{Connections, LocalConnections};
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_gateway::types::Session;
use ciproxy_github::app::webhook::HmacVerifier;
use ciproxy_github::app::{AppService, GitHubApp, InstallAuthority, LifecycleOutput};
use ciproxy_github::assignments::AssignmentService;
use ciproxy_github::auth::jwt::RsaAppJwt;
use ciproxy_github::auth::tokens::RotatingTokens;
use ciproxy_github::hooks::{AppWebhooks, WebhookHandler};
use ciproxy_github::installation_state::{AppInstallationClient, InstallationApi};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::redelivery::{HttpDeliveries, RecoverDeliveries, WebhookRecovery};
use ciproxy_github::registrations::RegistrationService;
use ciproxy_github::rest::http::RestClient;
use ciproxy_github::runtime::{Runtime, SystemRuntime};
use ciproxy_github::scale_sets::{ScaleSetFactory, sdk::AppScaleSetFactory};
use ciproxy_github::timing::RepairTiming;
use ciproxy_metering::service::StoredMetering;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::github_lifecycle::LifecycleEvent;
use ciproxy_store_interface::store::Store;
use reqwest::{Client, Url, redirect};
use secrecy::ExposeSecret;
use tokio::runtime::Handle;

use crate::accounting::{Accounting, AccountingWorker};
use crate::config::Config;
use crate::error::{Error, ErrorResultExt, Result};
use crate::image_signing::CdnSigning;
use crate::repositories::{RefreshRepositories, RepositoryRefresh, StoredInstallations};
use crate::runtime::{MachineRuntime, OwnMachines};

/// The process's shared gateway, GitHub App, and runtime service boundaries.
pub struct Hosted {
    /// Gateway used by the dashboard and its WebSocket transport.
    pub gateway: Arc<dyn MachineGateway>,
    /// App used by dashboard setup and repository changes.
    pub app: Arc<dyn GitHubApp>,
    /// Own-machine runtime used by listener notifications and reconciliation.
    pub machines: Arc<dyn OwnMachines>,
    pub(crate) store: Arc<dyn Store>,
    /// Scale-set operations used by the dashboard's label and take-out service.
    pub scale_sets: Arc<dyn ScaleSetFactory>,
    /// App signer shared with the API's installation administration adapter.
    pub signer: Arc<dyn ciproxy_github::auth::AppJwt>,
    pub(crate) rest: Arc<dyn ciproxy_github::rest::GitHubRest>,
    pub(crate) persistence: Arc<StoreAdapter>,
    transport: Arc<dyn AgentTransport>,
    hooks: Arc<dyn WebhookHandler>,
    pub(crate) ready: Arc<AtomicBool>,
    pub(crate) recovery: Arc<dyn WebhookRecovery>,
    pub(crate) repositories: Arc<dyn RepositoryRefresh>,
    pub(crate) accounting: Arc<dyn Accounting>,
}

impl Hosted {
    /// Bind real adapters to the sign-in service's existing authority seams.
    pub fn new(
        config: &Config,
        store: Arc<dyn Store>,
        authorization: Arc<dyn Authorization>,
        authority: Arc<dyn InstallAuthority>,
    ) -> Result<Self> {
        let clock: Arc<dyn Runtime> = Arc::new(SystemRuntime);
        let pem = config.secret("github-app-private-key")?;
        let signer = Arc::new(
            RsaAppJwt::from_pem(
                config.github_app_id.clone(),
                pem.expose_secret().as_bytes(),
                clock.clone(),
            )
            .internal()?,
        );
        let http = Client::builder()
            .redirect(redirect::Policy::none())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(90))
            .build()
            .internal()?;
        let api = Url::parse("https://api.github.com/").internal()?;
        let tokens = Arc::new(RotatingTokens::new(
            http.clone(),
            api.clone(),
            signer.clone(),
            clock.clone(),
        ));
        let rest = Arc::new(RestClient::new(http.clone(), api.clone(), tokens));
        let scale_sets: Arc<dyn ScaleSetFactory> =
            Arc::new(AppScaleSetFactory::new(signer.clone(), http.clone()));
        let persistence = Arc::new(StoreAdapter::new(store.clone(), clock.clone()));
        let relay = Arc::new(Relay {
            machines: OnceLock::new(),
            handle: Handle::current(),
        });
        let connections: Arc<dyn Connections> = Arc::new(LocalConnections::default());
        let gateway: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
            store.clone(),
            authorization,
            Arc::new(SystemClock),
            Arc::new(SystemEntropy),
            connections.clone(),
            relay.clone(),
        ));
        let installations: Arc<dyn InstallationApi> = Arc::new(AppInstallationClient::new(
            http.clone(),
            api.clone(),
            signer.clone(),
        ));
        let mut machine_runtime = MachineRuntime::new(
            store.clone(),
            gateway.clone(),
            Arc::new(RegistrationService {
                store: persistence.clone(),
                scale_sets: scale_sets.clone(),
            }),
            Arc::new(AssignmentService {
                store: persistence.clone(),
            }),
            clock.clone(),
            config.image.clone(),
            installations.clone(),
        );
        machine_runtime.image_signing = Some(Arc::new(CdnSigning::new(
            config.cdn_signing_key_name.clone(),
            config.secret("cdn-signing-key")?.expose_secret(),
        )?));
        let accounting = Arc::new(AccountingWorker::new(
            store.clone(),
            Arc::new(RepairTiming::new(
                persistence.clone(),
                rest.clone(),
                clock.clone(),
            )),
            Arc::new(StoredMetering::new(store.clone())),
            clock,
        ));
        let machines: Arc<dyn OwnMachines> = Arc::new(machine_runtime);
        relay
            .machines
            .set(Arc::downgrade(&machines))
            .map_or_else(|_| Err(Error::StateChanged), |_| Ok(()))?;
        let app: Arc<dyn GitHubApp> = Arc::new(AppService {
            leases: persistence.clone(),
            installations,
            store: persistence.clone(),
            outbox: persistence.clone(),
            rest: rest.clone(),
            scale_sets: scale_sets.clone(),
            authority,
            lifecycle: relay,
            runtime: Arc::new(SystemRuntime),
        });
        let hooks = Arc::new(AppWebhooks {
            verifier: Arc::new(HmacVerifier::new(config.secret("github-webhook-secret")?)),
            app: app.clone(),
        });
        let transport: Arc<dyn AgentTransport> =
            Arc::new(WebSocketTransport::new(gateway.clone(), connections));
        let repositories = Arc::new(RefreshRepositories {
            installations: Arc::new(StoredInstallations {
                store: store.clone(),
            }),
            app: app.clone(),
        });
        Ok(Self {
            accounting,
            repositories,
            gateway,
            app,
            machines,
            store,
            scale_sets,
            rest,
            persistence,
            transport,
            hooks,
            signer: signer.clone(),
            ready: Arc::new(AtomicBool::new(false)),
            recovery: Arc::new(RecoverDeliveries {
                api: Arc::new(HttpDeliveries {
                    http,
                    api,
                    signer: signer.clone(),
                }),
            }),
        })
    }

    /// Mount the dashboard service under /api on the same public host.
    pub fn router(&self, api: Router) -> Router {
        crate::http::router(
            self.transport.router(),
            api,
            self.hooks.clone(),
            self.ready.clone(),
        )
    }
}

struct Relay {
    machines: OnceLock<Weak<dyn OwnMachines>>,
    handle: Handle,
}

impl Relay {
    fn runtime(&self) -> Result<Arc<dyn OwnMachines>> {
        self.machines
            .get()
            .and_then(Weak::upgrade)
            .ok_or(Error::StateChanged)
    }
}

impl AgentEvents for Relay {
    fn machine_removed(
        &self,
        workspace_id: uuid::Uuid,
        machine_id: uuid::Uuid,
    ) -> ciproxy_gateway::error::Result<()> {
        let runtime = match self.runtime() {
            Ok(runtime) => runtime,
            Err(_) => return Err(ciproxy_gateway::error::Error::UnexpectedMessage),
        };
        self.handle
            .block_on(runtime.machine_removed(workspace_id, machine_id))
            .map_or_else(
                |_| Err(ciproxy_gateway::error::Error::UnexpectedMessage),
                |_| Ok(()),
            )
    }
    fn receive(
        &self,
        session: Session,
        message: AgentMessage,
    ) -> ciproxy_gateway::error::Result<()> {
        let result = self.runtime();
        match result {
            Ok(runtime) => self
                .handle
                .block_on(runtime.agent(session.workspace_id, session.machine_id, message))
                .map_or_else(
                    |_| Err(ciproxy_gateway::error::Error::UnexpectedMessage),
                    |_| Ok(()),
                ),
            Err(_) => Err(ciproxy_gateway::error::Error::UnexpectedMessage),
        }
    }
}

#[async_trait]
impl LifecycleOutput for Relay {
    async fn changed(&self, event: LifecycleEvent) -> ciproxy_github::error::Result<()> {
        let runtime = match self.runtime() {
            Ok(runtime) => runtime,
            Err(_) => return Err(ciproxy_github::error::Error::OrganizationInactive),
        };
        match runtime.lifecycle(event).await {
            Ok(()) => Ok(()),
            Err(_) => Err(ciproxy_github::error::Error::OrganizationInactive),
        }
    }
}
