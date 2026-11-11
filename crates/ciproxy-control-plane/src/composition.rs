//! Construct hosted adapters without exposing secrets to agents or API clients.

use std::sync::atomic::AtomicBool;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use axum::Router;
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_api::pages::PageServices;
use ciproxy_api::state::ApiState;
use ciproxy_gateway::boundary::{Authorization, SystemClock, SystemEntropy};
use ciproxy_gateway::connections::{Connections, LocalConnections};
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_gateway::transport::{AgentTransport, WebSocketTransport};
use ciproxy_github::app::webhook::HmacVerifier;
use ciproxy_github::app::{AppService, GitHubApp, InstallAuthority};
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
use ciproxy_metering::service::{Metering, StoredMetering};
use ciproxy_overflow::api::Overflow;
use ciproxy_store_interface::store::Store;
use reqwest::{Client, Url, redirect};
use secrecy::ExposeSecret;
use tokio::runtime::Handle;

use crate::accounting::{Accounting, AccountingWorker};
use crate::cache_metadata::records::StoredMetadataRecords;
use crate::cache_metadata::service::{CacheMetadata, HostedCacheMetadata};
use crate::cache_metadata::transport;
use crate::config::Config;
use crate::error::{Error, ErrorResultExt, Result};
use crate::hosted_events::Relay;
use crate::image_signing::CdnSigning;
use crate::repositories::{RefreshRepositories, RepositoryRefresh, StoredInstallations};
use crate::runtime::{MachineRuntime, OwnMachines};

/// The process's shared gateway, GitHub App, and runtime service boundaries.
pub struct Hosted {
    /// Shared Boat executor for runtime, callback, and dashboard routes.
    pub overflow: Option<Arc<dyn Overflow>>,
    /// The exact image sizing passed into the shared overflow executor.
    pub boat_root_sizing: Option<RootSizing>,
    /// Shared store-backed metering for accounting and dashboard reports.
    pub metering: Arc<dyn Metering>,
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
    metadata: Arc<dyn CacheMetadata>,
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
        let metadata = Arc::new(HostedCacheMetadata {
            records: Arc::new(StoredMetadataRecords {
                store: store.clone(),
            }),
            caches: rest.clone(),
            clock: clock.clone(),
        });
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
        let registrations = Arc::new(RegistrationService {
            store: persistence.clone(),
            scale_sets: scale_sets.clone(),
        });
        let overflow =
            crate::overflow_composition::compose(config, store.clone(), registrations.clone())?;
        let mut machine_runtime = MachineRuntime::new(
            store.clone(),
            gateway.clone(),
            registrations,
            Arc::new(AssignmentService {
                store: persistence.clone(),
            }),
            clock.clone(),
            config.image.clone(),
            installations.clone(),
        );
        machine_runtime.overflow = overflow.as_ref().map(|overflow| overflow.service.clone());
        machine_runtime.boat_retirement = overflow
            .as_ref()
            .map(|overflow| overflow.retirement.clone());
        machine_runtime.image_signing = Some(Arc::new(CdnSigning::new(
            config.cdn_signing_key_name.clone(),
            config.secret("cdn-signing-key")?.expose_secret(),
        )?));
        let metering: Arc<dyn Metering> = Arc::new(StoredMetering::new(store.clone()));
        let accounting = Arc::new(AccountingWorker::new(
            store.clone(),
            Arc::new(RepairTiming::new(
                persistence.clone(),
                rest.clone(),
                clock.clone(),
            )),
            metering.clone(),
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
            boat_root_sizing: overflow.as_ref().map(|overflow| overflow.root_sizing),
            overflow: overflow.map(|overflow| overflow.service),
            metering,
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
            metadata,
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
    pub fn router(&self, state: ApiState, pages: Arc<PageServices>) -> Router {
        let router = crate::http::dashboard_router(
            self.transport.router(),
            state,
            pages,
            self.hooks.clone(),
            self.ready.clone(),
        );
        let router = router.merge(transport::router(self.metadata.clone()));
        if let Some(overflow) = &self.overflow {
            router.merge(crate::boat_callback::router(overflow.clone()))
        } else {
            router
        }
    }
}
