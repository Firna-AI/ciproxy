//! JIT wire configuration and registration removal use the real SDK.

use std::sync::Arc;
use std::sync::atomic::AtomicI64;

use ciproxy_github::persistence::StoreAdapter;
use ciproxy_github::registrations::{RegistrationService, Removal, RunnerRegistrations};
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
    let registration = service.create(target, "runner-71").await.unwrap();
    assert_eq!(registration.runner_id, 71);
    assert_eq!(
        registration.configuration.expose_secret(),
        "test-one-use-configuration"
    );
    assert!(!format!("{registration:?}").contains("test-one-use-configuration"));
    assert!(matches!(
        service.remove(target, 71).await,
        Ok(Removal::Removed)
    ));
    assert!(matches!(
        service.remove(target, 71).await,
        Ok(Removal::Removed)
    ));
    assert!(matches!(
        service.remove(target, 73).await,
        Ok(Removal::Busy)
    ));
    let requests = server.state.requests.lock().unwrap();
    let jit = requests
        .iter()
        .find(|(method, path, _)| *method == Method::POST && path.contains("/53/generatejitconfig"))
        .unwrap();
    assert_eq!(jit.2, json!({"name":"runner-71","workFolder":""}));
}
