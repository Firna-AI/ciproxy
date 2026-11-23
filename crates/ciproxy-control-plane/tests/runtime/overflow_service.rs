//! Shared real-store overflow composition with replaceable GitHub boundaries.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_control_plane::overflow_authority::OverflowRunnerAuthority;
use ciproxy_control_plane::overflow_callback::EXIT_PATH;
use ciproxy_control_plane::runtime::MachineRuntime;
use ciproxy_overflow::api::{Overflow, OverflowConfig};
use ciproxy_overflow::registry::{ImageSource, ProviderRegistry};
use ciproxy_overflow::service::{OverflowDependencies, OverflowService};
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::provider::OverflowProvider;
use tower::ServiceExt;

use super::overflow_ports::{Factory, Ports};
use super::support::Fixture;

pub(super) fn attach(
    runtime: &mut MachineRuntime,
    fixture: &Fixture,
    ports: &Arc<Ports>,
    root_sizing: RootSizing,
) -> Arc<dyn Overflow> {
    let providers = Arc::new(ProviderRegistry::default().with_provider(
        OverflowProvider::Boat,
        Arc::new(Factory(ports.clone())),
        ImageSource {
            reference: "shared-image".to_owned(),
            root_sizing,
        },
    ));
    let authority = Arc::new(OverflowRunnerAuthority {
        store: fixture.store.clone(),
        registrations: runtime.registrations.clone(),
        root_sizing: RootSizing::default(),
        providers: providers.clone(),
        clock: ports.clone(),
    });
    let service: Arc<dyn Overflow> = Arc::new(OverflowService::new(
        OverflowConfig {
            deployment: "runtime-integration".to_owned(),
            exit_report_url: Secret::new(format!("https://app.ciproxy.example{EXIT_PATH}")),
        },
        OverflowDependencies {
            store: fixture.store.clone(),
            cipher: ports.clone(),
            providers: providers.clone(),
            runners: authority.clone(),
            clock: ports.clone(),
            tokens: ports.clone(),
        },
    ));
    runtime.overflow = Some(service.clone());
    runtime.overflow_retirement = Some(authority);
    runtime.providers = providers;
    service
}

pub(super) async fn post_exit_at(
    router: &Router,
    path: &str,
    token: &str,
    runner: &str,
) -> StatusCode {
    let request = Request::builder()
        .method("POST")
        .uri(path)
        .header("authorization", format!("Bearer {token}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"type":"runner_exited","runner_id":runner,"exit":{"code":0}})
                .to_string(),
        ))
        .unwrap();
    router.clone().oneshot(request).await.unwrap().status()
}
