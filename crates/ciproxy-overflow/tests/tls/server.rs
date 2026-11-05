//! Shared verified TLS server with exact provider request consumption.

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use ciproxy_overflow::boat::BoatFactory;
use ciproxy_overflow::provider::{BackendFactory, Capabilities};
use ciproxy_protocol::secret::Secret;
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{ServerConfig, crypto::aws_lc_rs, pki_types::PrivatePkcs8KeyDer},
};

use super::wire::{self, Request};

pub(crate) struct Step {
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) status: u16,
    pub(crate) body: Value,
}

pub(crate) struct Server {
    pub(crate) backend: Capabilities,
    state: Arc<Mutex<(VecDeque<Step>, Vec<Request>)>>,
    task: JoinHandle<()>,
}

impl Server {
    pub(crate) async fn new(steps: Vec<Step>) -> Self {
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

    pub(crate) fn captured(&self) -> Vec<Request> {
        self.state.lock().unwrap().1.clone()
    }
    pub(crate) fn assert_consumed(&self) {
        assert_eq!(self.state.lock().unwrap().0.len(), 0);
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

pub(crate) fn step(method: &'static str, path: &'static str, status: u16, body: Value) -> Step {
    Step {
        method,
        path,
        status,
        body,
    }
}
