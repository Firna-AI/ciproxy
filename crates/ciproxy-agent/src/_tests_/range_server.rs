//! A local HTTP image host that answers each request by a script, and
//! records the `Range` header of each request.

use std::sync::{Arc, Mutex};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

/// The answer to one request.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Reply {
    /// `206` from the requested start, or `200` without a `Range` header.
    Serve,
    /// As `Serve`, but close the connection after this many body bytes.
    Break(usize),
    /// `200` with the whole body, also for a `Range` request.
    IgnoreRange,
    /// `206` whose `Content-Range` starts one byte after the request.
    Shifted,
    /// This status with no body.
    Status(u16),
    /// `429` with `Retry-After: 30` and no body.
    RateLimit,
}

/// One running host for one body.
pub(crate) struct RangeServer {
    pub(crate) url: String,
    ranges: Arc<Mutex<Vec<Option<String>>>>,
    task: JoinHandle<()>,
}

impl RangeServer {
    /// Serve `body` with one reply for each request, in order.
    pub(crate) async fn start(body: &[u8], replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}/macos.tar", listener.local_addr().unwrap());
        let ranges = Arc::new(Mutex::new(Vec::new()));
        let (body, seen) = (body.to_vec(), ranges.clone());
        let task = tokio::spawn(async move {
            for reply in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let range = read_range(&mut socket).await;
                seen.lock()
                    .unwrap()
                    .push(range.map(|start| format!("bytes={start}-")));
                answer(&mut socket, &body, range, reply).await;
            }
        });
        Self { url, ranges, task }
    }

    /// The `Range` header of each request so far, in order.
    pub(crate) fn ranges(&self) -> Vec<Option<String>> {
        self.ranges.lock().unwrap().clone()
    }
}

impl Drop for RangeServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// Read the request head, and give the start of its `Range: bytes=N-`.
async fn read_range(socket: &mut TcpStream) -> Option<usize> {
    let mut head = Vec::new();
    while !head.ends_with(b"\r\n\r\n") {
        head.push(socket.read_u8().await.unwrap());
    }
    let head = String::from_utf8(head).unwrap();
    head.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        let value = value.trim();
        name.eq_ignore_ascii_case("range").then(|| {
            value
                .strip_prefix("bytes=")?
                .strip_suffix('-')?
                .parse()
                .ok()
        })?
    })
}

async fn answer(socket: &mut TcpStream, body: &[u8], range: Option<usize>, reply: Reply) {
    let from = |shift: usize| match range {
        Some(start) => (206, Some(start + shift), &body[start..]),
        None => (200, None, body),
    };
    let ((status, shown, rest), cut) = match reply {
        Reply::Status(status) => ((status, None, &body[..0]), None),
        Reply::RateLimit => ((429, None, &body[..0]), None),
        Reply::IgnoreRange => ((200, None, body), None),
        Reply::Serve => (from(0), None),
        Reply::Break(bytes) => (from(0), Some(bytes)),
        Reply::Shifted => (from(1), None),
    };
    let mut head = format!(
        "HTTP/1.1 {status} Answer\r\nContent-Length: {}\r\nConnection: close\r\n",
        rest.len()
    );
    if let Some(shown) = shown {
        let last = body.len() - 1;
        head.push_str(&format!(
            "Content-Range: bytes {shown}-{last}/{}\r\n",
            body.len()
        ));
    }
    if let Reply::RateLimit = reply {
        head.push_str("Retry-After: 30\r\n");
    }
    head.push_str("\r\n");
    socket.write_all(head.as_bytes()).await.unwrap();
    socket
        .write_all(&rest[..cut.unwrap_or(rest.len())])
        .await
        .unwrap();
    socket.shutdown().await.unwrap();
}
