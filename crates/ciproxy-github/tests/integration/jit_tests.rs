//! JIT wire configuration and registration removal use the real SDK.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::error::Error;
use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{
    RegistrationRequest, RegistrationRetirement, RegistrationService, RunnerRegistrations,
};
use ciproxy_github::scale_sets::ScaleSetFactory;
use ciproxy_store_interface::scale_sets::RegistrationTarget;
use reqwest::Method;
use secrecy::ExposeSecret;
use serde_json::json;

use super::database::Database;
use super::fake_http::Server;
use super::support::{Factory, FixedRuntime, setup};

#[tokio::test]
async fn jit_http_registers_one_runner_and_removal_accepts_missing_but_preserves_busy() {
    let database = Database::new();
    let target = setup(&*database.store);
    let server = Server::start(None).await;
    let runtime = Arc::new(FixedRuntime(AtomicI64::new(1_793_793_600)));
    let service = RegistrationService {
        store: Arc::new(StoreAdapter::new(database.store.clone(), runtime)),
        scale_sets: Arc::new(Factory(server.origin.clone())),
    };
    let context = service.store.context(target).await.unwrap();
    let request = RegistrationRequest {
        workspace_id: target.workspace_id,
        target: RegistrationTarget::from(&context.scale_set),
    };
    let registration = service.create(request, "runner-71").await.unwrap();
    assert_eq!(registration.runner_id, 71);
    assert_eq!(registration.target.github_id, 53);
    assert_eq!(registration.target.scale_set_id, target.scale_set_id);
    assert_eq!(
        registration.configuration.expose_secret(),
        "test-one-use-configuration"
    );
    assert!(!format!("{registration:?}").contains("test-one-use-configuration"));
    assert!(matches!(
        service.retire(target, "runner-71", Some(71)).await,
        Ok(RegistrationRetirement::Removed { id: 71 })
    ));
    assert!(matches!(
        service.retire(target, "runner-71", Some(71)).await,
        Ok(RegistrationRetirement::Removed { id: 71 })
    ));
    assert!(matches!(
        service.retire(target, "runner-73", Some(73)).await,
        Ok(RegistrationRetirement::Busy { id: 73 })
    ));
    let requests = server.state.requests.lock().unwrap().clone();
    let jit = requests
        .iter()
        .find(|(method, path, _)| *method == Method::POST && path.contains("/53/generatejitconfig"))
        .unwrap();
    assert_eq!(jit.2, json!({"name":"runner-71","workFolder":""}));
    for route in ["/access_tokens", "/registration-token"] {
        *server.state.resource.auth_failure.lock().unwrap() = Some(route.to_owned());
        assert!(
            matches!(
                service.retire(target, "runner-73", Some(73)).await,
                Err(Error::Internal(_))
            ),
            "{route}: auth failure must not claim a busy runner was removed"
        );
        assert!(
            matches!(
                service.retire(target, "unknown-runner", None).await,
                Err(Error::Internal(_))
            ),
            "{route}: auth failure must not claim name lookup found no runner"
        );
        assert!(
            matches!(
                service.create(request, "still-busy").await,
                Err(Error::Internal(_))
            ),
            "{route}: auth failure must not claim the scale set is missing"
        );
        let context = service.store.context(target).await.unwrap();
        let api = Factory(server.origin.clone())
            .for_organization(&context.organization)
            .await
            .unwrap();
        assert!(
            matches!(api.open(53, "auth-failure").await, Err(Error::Internal(_))),
            "{route}: auth failure must not request scale-set replacement"
        );
        assert!(
            matches!(
                api.rename_scale_set(53, "ciproxy-retired-53").await,
                Err(Error::Internal(_))
            ),
            "{route}: auth failure must not claim a retired name was released"
        );
    }
}
