//! A local HTTPS store with a self-signed certificate for 127.0.0.1.

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::serve::Listener;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{ServerConfig, crypto::aws_lc_rs, pki_types::PrivatePkcs8KeyDer},
};

/// The bound store and the certificate that a client must trust.
pub(crate) struct TlsStore {
    /// Local address of the store.
    pub(crate) address: SocketAddr,
    /// PEM of the self-signed store certificate.
    pub(crate) certificate: String,
}

struct TlsListener {
    tcp: TcpListener,
    acceptor: TlsAcceptor,
}

impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let Ok((socket, address)) = self.tcp.accept().await else {
                continue;
            };
            if let Ok(stream) = self.acceptor.accept(socket).await {
                return (stream, address);
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}

/// Serve the routes over HTTPS until the test runtime ends.
pub(crate) async fn serve(routes: Router) -> TlsStore {
    let identity = rcgen::generate_simple_self_signed(vec!["127.0.0.1".to_owned()]).unwrap();
    let config = ServerConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![identity.cert.der().clone()],
            PrivatePkcs8KeyDer::from(identity.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = tcp.local_addr().unwrap();
    let listener = TlsListener {
        tcp,
        acceptor: TlsAcceptor::from(Arc::new(config)),
    };
    tokio::spawn(async move { axum::serve(listener, routes).await.unwrap() });
    TlsStore {
        address,
        certificate: identity.cert.pem(),
    }
}
