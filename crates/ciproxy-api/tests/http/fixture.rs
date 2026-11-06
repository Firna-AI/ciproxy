//! Actual router, gateway, AppService, and migrated store for each HTTP scenario.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::http::{Request, Response, StatusCode, header};
use chrono::Duration;
use ciproxy_api::access::AccessService;
use ciproxy_api::crypto::AeadCredentialCipher;
use ciproxy_api::router::router;
use ciproxy_api::runtime::Runtime;
use ciproxy_api::state::{ApiConfig, ApiState};
use ciproxy_gateway::boundary::{AgentEvents, SystemEntropy};
use ciproxy_gateway::connections::LocalConnections;
use ciproxy_gateway::gateway::{Gateway, MachineGateway};
use ciproxy_github::app::{AppService, GitHubApp};
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_store_interface::dashboard::records::{Membership, Role};
use ciproxy_store_interface::labels::{Label, WhenNothingFits};
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use ciproxy_store_interface::scale_sets::ScaleSet;
use ciproxy_store_interface::vm_size::VmSize;
use ciproxy_store_interface::workspaces::Workspace;
use serde_json::Value;
use tower::ServiceExt;
use unimock::Unimock;
use url::Url;
use uuid::Uuid;

use super::database::Database;
use super::fixture_io::{CallbackOutcome, callback, cookie, oauth_state};
use super::ports::{Factory, Identity, Inputs, Remote};

struct Events;
impl AgentEvents for Events {
    fn receive(
        &self,
        _: ciproxy_gateway::types::Session,
        _: AgentMessage,
    ) -> ciproxy_gateway::error::Result<()> {
        Ok(())
    }
}

pub(super) struct Browser {
    pub(super) cookie: String,
    pub(super) csrf: String,
}

pub(super) struct Fixture {
    pub(super) database: Database,
    pub(super) state: ApiState,
    pub(super) identity: Arc<Identity>,
    pub(super) runtime: Arc<Inputs>,
    pub(super) remote: Arc<Remote>,
    pub(super) gateway: Arc<dyn MachineGateway>,
    pub(super) app: Arc<dyn GitHubApp>,
    pub(super) workspace: Uuid,
    pub(super) organization: Organization,
}

impl Fixture {
    pub(super) fn new() -> Self {
        let database = Database::new();
        let runtime = Arc::new(Inputs(Mutex::new("2026-11-06T12:00:00Z".parse().unwrap())));
        let identity = Arc::new(Identity {
            id: AtomicI64::new(7),
            memberships: Mutex::new(HashMap::new()),
            fail: AtomicBool::new(false),
        });
        let access = Arc::new(AccessService::new(
            database.store.clone(),
            identity.clone(),
            Arc::new(AeadCredentialCipher::new(&[17; 32])),
            runtime.clone(),
        ));
        let gateway: Arc<dyn MachineGateway> = Arc::new(Gateway::new(
            database.store.clone(),
            access.clone(),
            runtime.clone(),
            Arc::new(SystemEntropy),
            Arc::new(LocalConnections::default()),
            Arc::new(Events),
        ));
        let remote = Arc::new(Remote {
            store: database.store.clone(),
            organizations: Mutex::new(Vec::new()),
            sets: Mutex::new(HashMap::new()),
            next_id: AtomicI64::new(1000),
            events: Mutex::new(Vec::new()),
            fail_uninstall: AtomicBool::new(false),
            fail_delete: AtomicBool::new(false),
            fail_lifecycle: AtomicBool::new(false),
        });
        let factory = Arc::new(Factory(remote.clone()));
        let persistence = Arc::new(StoreAdapter::new(database.store.clone(), runtime.clone()));
        let app: Arc<dyn GitHubApp> = Arc::new(AppService {
            leases: persistence.clone(),
            installations: remote.clone(),
            store: persistence.clone(),
            outbox: persistence,
            rest: remote.clone(),
            scale_sets: factory.clone(),
            authority: access.clone(),
            lifecycle: remote.clone(),
            runtime: runtime.clone(),
        });
        let state = ApiState {
            capacity: Arc::new(Unimock::new(())),
            config: ApiConfig {
                public_origin: "https://app.ciproxy.example".to_owned(),
                github_install_url: "https://github.example/apps/ciproxy/installations/new"
                    .to_owned(),
                download_url: "https://get.ciproxy.example".to_owned(),
                image_disk_bytes: 58_300_000_000,
                build: "http-test-build".to_owned(),
            },
            store: database.store.clone(),
            access,
            gateway: gateway.clone(),
            github: app.clone(),
            scale_sets: factory,
            installations: remote.clone(),
            runtime: runtime.clone(),
        };
        let workspace = Uuid::new_v4();
        database
            .store
            .create_workspace(&Workspace {
                id: workspace,
                name: "Customer".to_owned(),
                delete_after: None,
            })
            .unwrap();
        let organization = Organization {
            id: Uuid::new_v4(),
            workspace_id: workspace,
            github_id: 100,
            login: "customer".to_owned(),
            installation_id: 101,
            app_state: AppState::Active,
            repositories: RepositoryAccess::AllPrivate,
            removed_at: None,
            delete_after: None,
        };
        database.store.install_organization(&organization).unwrap();
        remote
            .organizations
            .lock()
            .unwrap()
            .push(organization.clone());
        let label = Label {
            id: Uuid::new_v4(),
            name: "ciproxy".to_owned(),
            size: VmSize {
                vcpus: 2,
                memory_gb: 8,
                disk_gb: 20,
            },
            allow_machines: true,
            allow_boat: true,
            when_nothing_fits: WhenNothingFits::Wait,
        };
        database.store.create_label(workspace, &label).unwrap();
        database
            .store
            .create_scale_set(
                workspace,
                &ScaleSet {
                    id: Uuid::new_v4(),
                    organization_id: organization.id,
                    label_id: label.id,
                    github_id: 501,
                    runner_group_id: 47,
                },
            )
            .unwrap();
        remote.sets.lock().unwrap().insert((label.name, 47), 501);
        Self {
            database,
            state,
            identity,
            runtime,
            remote,
            gateway,
            app,
            workspace,
            organization,
        }
    }

    pub(super) async fn request(
        &self,
        browser: Option<&Browser>,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Response<Body> {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(browser) = browser {
            request = request.header(header::COOKIE, format!("__Host-ciproxy={}", browser.cookie));
            if method != "GET" {
                request = request
                    .header(header::ORIGIN, &self.state.config.public_origin)
                    .header("x-ciproxy-csrf", &browser.csrf);
            }
        }
        let request = if let Some(body) = body {
            request
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap()
        } else {
            request.body(Body::empty()).unwrap()
        };
        let response = router(self.state.clone()).oneshot(request).await.unwrap();
        assert_eq!(
            response.headers().get("x-ciproxy-build").unwrap(),
            "http-test-build"
        );
        response
    }

    pub(super) async fn login(&self, github: i64, role: Option<Role>) -> Browser {
        self.identity.id.store(github, Ordering::SeqCst);
        self.identity
            .memberships
            .lock()
            .unwrap()
            .entry(github)
            .or_insert_with(|| {
                vec![Membership {
                    github_id: 100,
                    owner: false,
                }]
            });
        if role == Some(Role::Admin) {
            self.database
                .store
                .ensure_dashboard_initial_admin(self.workspace, github, self.runtime.now())
                .unwrap();
        }
        let begin = self.request(None, "GET", "/auth/github", None).await;
        assert_eq!(begin.status(), StatusCode::SEE_OTHER);
        let binding = cookie(&begin, "__Host-ciproxy-oauth");
        let state = oauth_state(&begin);
        let location = Url::parse(
            begin
                .headers()
                .get(header::LOCATION)
                .unwrap()
                .to_str()
                .unwrap(),
        )
        .unwrap();
        assert!(
            location
                .query_pairs()
                .any(|(key, value)| key == "code_challenge" && value.len() > 40)
        );
        let request = Request::builder()
            .uri(format!(
                "/auth/github/callback?code=synthetic&state={state}"
            ))
            .header(header::COOKIE, format!("__Host-ciproxy-oauth={binding}"))
            .body(Body::empty())
            .unwrap();
        let response = callback(
            router(self.state.clone()),
            request,
            CallbackOutcome::Accepted,
        )
        .await;
        let browser = Browser {
            cookie: cookie(&response, "__Host-ciproxy"),
            csrf: cookie(&response, "__Host-ciproxy-csrf"),
        };
        if let Some(role) = role {
            self.database
                .store
                .save_dashboard_role(self.workspace, github, Some(role), self.runtime.now())
                .unwrap();
        }
        browser
    }

    pub(super) fn path(&self, suffix: &str) -> String {
        format!("/workspaces/{}{suffix}", self.workspace)
    }
    pub(super) fn advance(&self, duration: Duration) {
        self.runtime.advance(duration);
    }
}
