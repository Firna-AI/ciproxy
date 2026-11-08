//! Forward real PostgreSQL queries while holding client Terminate messages.

use std::io::{self, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use url::Url;

pub(super) struct HeldDisconnect {
    url: String,
    address: SocketAddr,
    stopped: Arc<AtomicBool>,
    disconnected: Receiver<()>,
    worker: Option<JoinHandle<()>>,
}

impl HeldDisconnect {
    pub(super) fn new(database_url: &str) -> Self {
        let mut url = Url::parse(database_url).unwrap();
        let host = url.host_str().unwrap().to_owned();
        let port = url.port().unwrap_or(5432);
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        url.set_host(Some("127.0.0.1")).unwrap();
        url.set_port(Some(address.port())).unwrap();
        url.query_pairs_mut().append_pair("sslmode", "disable");
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stopped);
        let (sender, disconnected) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut connections = Vec::new();
            let mut workers = Vec::new();
            for accepted in listener.incoming() {
                let client = accepted.unwrap();
                if stop.load(Ordering::SeqCst) {
                    break;
                }
                let server = TcpStream::connect((host.as_str(), port)).unwrap();
                client.set_nodelay(true).unwrap();
                server.set_nodelay(true).unwrap();
                connections.push(client.try_clone().unwrap());
                connections.push(server.try_clone().unwrap());
                let sender = sender.clone();
                workers.push(thread::spawn(move || forward(client, server, sender)));
            }
            for connection in connections {
                let _ = connection.shutdown(Shutdown::Both);
            }
            for worker in workers {
                worker.join().unwrap();
            }
        });
        Self {
            url: url.to_string(),
            address,
            stopped,
            disconnected,
            worker: Some(worker),
        }
    }

    pub(super) fn url(&self) -> &str {
        &self.url
    }

    pub(super) fn wait_for_disconnect(&self) {
        self.disconnected
            .recv_timeout(Duration::from_secs(10))
            .expect("billing connection must send Terminate");
    }
}

impl Drop for HeldDisconnect {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        TcpStream::connect(self.address).unwrap();
        self.worker.take().unwrap().join().unwrap();
    }
}

fn forward(mut client: TcpStream, mut server: TcpStream, disconnected: Sender<()>) {
    let mut response_source = server.try_clone().unwrap();
    let mut response_target = client.try_clone().unwrap();
    let responses = thread::spawn(move || {
        let _ = io::copy(&mut response_source, &mut response_target);
    });
    let mut startup = [0; 4];
    client.read_exact(&mut startup).unwrap();
    let mut body = vec![0; u32::from_be_bytes(startup) as usize - 4];
    client.read_exact(&mut body).unwrap();
    server.write_all(&startup).unwrap();
    server.write_all(&body).unwrap();
    let mut header = [0; 5];
    while client.read_exact(&mut header).is_ok() {
        let length = u32::from_be_bytes(header[1..].try_into().unwrap());
        let mut body = vec![0; length as usize - 4];
        client.read_exact(&mut body).unwrap();
        if header[0] == b'X' {
            disconnected.send(()).unwrap();
            break;
        }
        server.write_all(&header).unwrap();
        server.write_all(&body).unwrap();
    }
    responses.join().unwrap();
}
