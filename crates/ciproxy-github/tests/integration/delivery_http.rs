//! Dedicated App delivery HTTP peer with required App JWT authentication.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderName, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

pub(super) struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: String,
}

impl Reply {
    pub(super) fn new(status: StatusCode, body: &str, headers: &[(&str, &str)]) -> Self {
        let mut values = HeaderMap::new();
        values.insert("content-type", "application/json".parse().unwrap());
        for (name, value) in headers {
            values.insert(
                HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
        }
        Self {
            status,
            headers: values,
            body: body.to_owned(),
        }
    }
}

#[derive(Default)]
struct Capture {
    replies: VecDeque<Reply>,
    requests: Vec<(Method, String)>,
}

pub(super) struct Peer {
    pub(super) origin: String,
    capture: Arc<Mutex<Capture>>,
    task: JoinHandle<()>,
}

impl Peer {
    pub(super) async fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let capture = Arc::new(Mutex::new(Capture {
            replies: replies.into(),
            ..Capture::default()
        }));
        let router = Router::new().fallback(handle).with_state(capture.clone());
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        Self {
            origin,
            capture,
            task,
        }
    }

    pub(super) fn requests(&self) -> Vec<(Method, String)> {
        self.capture.lock().unwrap().requests.clone()
    }
}

impl Drop for Peer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn handle(
    State(capture): State<Arc<Mutex<Capture>>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    let mut capture = capture.lock().unwrap();
    capture.requests.push((method, uri.to_string()));
    if !headers
        .get("authorization")
        .is_some_and(|value| value == "Bearer test-app-jwt")
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let Some(reply) = capture.replies.pop_front() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    (reply.status, reply.headers, reply.body).into_response()
}
