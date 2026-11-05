//! Scripted verified TLS provider fixture with exact request consumption.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::provider::{BackendFactory, Capabilities};
use ciproxy_protocol::secret::Secret;
use sandbox_interface::{
    BackendCreateSandboxRequest, OperationId, ProviderRef, ResourceOwner, SandboxConsumer,
    SandboxId, SandboxLifetime, SandboxNetworkPolicy, SandboxSizeSelection,
};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{ServerConfig, crypto::aws_lc_rs, pki_types::PrivatePkcs8KeyDer},
};
use uuid::Uuid;

use super::wire::{self, Request};

pub(super) struct Step {
    pub(super) method: &'static str,
    pub(super) path: &'static str,
    pub(super) status: u16,
    pub(super) body: Value,
}

pub(super) struct Server {
    pub(super) backend: Capabilities,
    state: Arc<Mutex<(VecDeque<Step>, Vec<Request>)>>,
    task: JoinHandle<()>,
}

impl Server {
    pub(super) async fn new(steps: Vec<Step>) -> Self {
        let identity = rcgen::generate_simple_self_signed(vec![
            "localhost".to_owned(),
            "127.0.0.1".to_owned(),
        ])
        .unwrap();
        let config = ServerConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(
                vec![identity.cert.der().clone()],
                PrivatePkcs8KeyDer::from(identity.signing_key.serialize_der()).into(),
            )
            .unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let factory = BoatFactory::with_origin(
            format!(
                "https://127.0.0.1:{}",
                listener.local_addr().unwrap().port()
            ),
            Some(identity.cert.pem().into_bytes()),
        );
        let backend = factory
            .backend(&Secret::new("fixture-key"), "personal")
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));
        let state = Arc::new(Mutex::new((VecDeque::from(steps), Vec::new())));
        let server_state = state.clone();
        let task = tokio::spawn(async move {
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let mut stream = acceptor.accept(socket).await.unwrap();
                let request = wire::read(&mut stream).await;
                let step = {
                    let mut state = server_state.lock().unwrap();
                    let step = state.0.pop_front().expect("unexpected provider request");
                    assert_eq!(request.method, step.method);
                    assert!(request.path.starts_with(step.path), "{}", request.path);
                    assert_eq!(request.headers["authorization"], "Bearer fixture-key");
                    assert_eq!(request.headers["x-boat-org"], "personal");
                    state.1.push(request);
                    step
                };
                if step.status != 0 {
                    wire::reply(&mut stream, step.status, step.body).await;
                }
            }
        });
        Self {
            backend,
            state,
            task,
        }
    }

    pub(super) fn captured(&self) -> Vec<Request> {
        self.state.lock().unwrap().1.clone()
    }
    pub(super) fn assert_consumed(&self) {
        assert_eq!(self.state.lock().unwrap().0.len(), 0);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(super) fn step(method: &'static str, path: &'static str, status: u16, body: Value) -> Step {
    Step {
        method,
        path,
        status,
        body,
    }
}
pub(super) fn limits(can_start: bool) -> Value {
    json!({"ok":true,"type":"limits.info","canStart":can_start,"billingStatus":"active","sandboxPlanKey":"box_20", "hasPaymentHistory":true,"nextStartDollars":0.03,
        "machineTypes":[{"type":"large","available":true,"reason":null}]})
}
pub(super) fn identity() -> Value {
    json!({"ok":true,"type":"user.info","user":{"login":"customer","displayName":"Customer Boat"}})
}
pub(super) fn request() -> BackendCreateSandboxRequest {
    BackendCreateSandboxRequest {
        sandbox_id: SandboxId::from_uuid(Uuid::from_u128(12)),
        operation_id: OperationId::from_uuid(Uuid::from_u128(11)),
        owner: ResourceOwner::platform(Uuid::from_u128(1)),
        consumer: SandboxConsumer::Runtime,
        lifetime: SandboxLifetime::OneShot {
            max_lifetime: std::time::Duration::from_secs(21600),
        },
        size: Some(SandboxSizeSelection::ProviderSize("large".to_owned())),
        deployment_id: "ciproxy-live-fixture/workspace-1".to_owned(),
        profile: "ciproxy".to_owned(),
        network: SandboxNetworkPolicy::Open,
        snapshot_provider_ref: Some(ProviderRef::new("shared-image-v7")),
    }
}
