//! Fake upstream HTTP preserves the production OAuth and SDK error paths.

use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;
use uuid::Uuid;

use super::oauth_peer::OAuthPeer;
use super::setup_peer::{self, Control};

pub(super) struct Server {
    pub(super) origin: String,
    pub(super) token: SecretString,
    pub(super) seen: Arc<Mutex<Vec<String>>>,
    pub(super) control: Arc<Control>,
    pub(super) oauth: Arc<OAuthPeer>,
    task: JoinHandle<()>,
}
struct Data {
    origin: String,
    token: SecretString,
    seen: Arc<Mutex<Vec<String>>>,
    control: Arc<Control>,
    oauth: Arc<OAuthPeer>,
}
impl Server {
    pub(super) async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}/", listener.local_addr().unwrap());
        let token: SecretString = Uuid::new_v4().to_string().into();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let control = Arc::new(Control::new());
        let oauth = Arc::new(OAuthPeer::new(token.clone()));
        let state = Arc::new(Data {
            origin: origin.clone(),
            token: token.clone(),
            seen: seen.clone(),
            control: control.clone(),
            oauth: oauth.clone(),
        });
        let task = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(handle).with_state(state))
                .await
                .unwrap();
        });
        Self {
            origin,
            token,
            seen,
            control,
            oauth,
            task,
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn handle(
    State(data): State<Arc<Data>>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    let path = uri.path();
    data.seen.lock().unwrap().push(format!("{method} {path}"));
    if path == "/login/oauth/access_token" {
        let form = Url::parse(&format!(
            "http://fake/?{}",
            String::from_utf8(bytes.to_vec()).unwrap()
        ))
        .unwrap();
        let fields = form
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<std::collections::HashMap<_, _>>();
        if !fields.contains_key("client_secret") {
            return StatusCode::BAD_REQUEST.into_response();
        }
        return data.oauth.token(&fields);
    }
    if path == "/user" {
        return Json(
            json!({"id":data.control.person.load(Ordering::SeqCst),"login":format!("person-{}",data.control.person.load(Ordering::SeqCst)),"avatar_url":"https://github.example/avatar"}),
        )
        .into_response();
    }
    let url = Url::parse(&format!("http://fake{uri}")).unwrap();
    let page = url
        .query_pairs()
        .find(|(key, _)| key == "page")
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| "1".to_owned());
    if let Some(response) = setup_peer::respond(&data.control, path, &page) {
        return response;
    }
    if path == "/user/memberships/orgs" {
        if let Some(error) = data.oauth.membership_error(&headers) {
            return error;
        }
        if !data.control.memberships.load(Ordering::SeqCst) {
            return Json(json!([])).into_response();
        }
        if let Some(ids) = &*data.control.membership_ids.lock().unwrap() {
            return Json(
                ids.iter()
                    .skip((page.parse::<usize>().unwrap() - 1) * 100)
                    .take(100)
                    .map(|id| json!({"state":"active","role":"member","organization":{"id":id}}))
                    .collect::<Vec<_>>(),
            )
            .into_response();
        }
        let batch = if page == "1" {
            (0..100).map(|index|json!({"state":"active","role":if index==0 {"admin"}else{"member"},"organization":{"id":100+index}})).collect::<Vec<_>>()
        } else {
            vec![
                json!({"state":"pending","role":"admin","organization":{"id":999}}),
                json!({"state":"active","role":"member","organization":{"id":200}}),
            ]
        };
        return Json(batch).into_response();
    }
    if path == "/app/installation-requests" {
        let requests = if page == "1" {
            (0..100).map(|index|json!({"id":index,"requester":{"id":8},"account":{"id":index+500,"login":"other","type":"Organization"},"created_at":"2026-11-06T11:00:00Z"})).collect::<Vec<_>>()
        } else {
            vec![
                json!({"id":1000,"requester":{"id":7},"account":{"id":900,"login":"waiting-org","type":"Organization"},"created_at":"2026-11-06T11:30:00Z"}),
            ]
        };
        return Json(requests).into_response();
    }
    if path.ends_with("/access_tokens") {
        return (
            StatusCode::CREATED,
            Json(json!({"token":data.token.expose_secret(),"expires_at":"2099-01-01T00:00:00Z"})),
        )
            .into_response();
    }
    if path.ends_with("/registration-token") {
        return (
            StatusCode::CREATED,
            Json(json!({"token":data.token.expose_secret()})),
        )
            .into_response();
    }
    if path.ends_with("/runner-registration") {
        return Json(json!({"url":format!("{}actions",data.origin),"token":"e30.eyJleHAiOjQxMDI0NDQ4MDB9.synthetic"})).into_response();
    }
    if path.ends_with("/runnerscalesets") && method == Method::GET {
        return Json(json!({"count":0,"value":[]})).into_response();
    }
    if path.ends_with("/runnerscalesets") && method == Method::POST {
        return (StatusCode::UNPROCESSABLE_ENTITY,Json(json!({"typeName":"RunnerScaleSetValidationException","message":"GitHub refused this exact label name."}))).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}
