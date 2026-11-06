//! Fake upstream HTTP preserves the production OAuth and SDK error paths.

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

pub(super) struct Server {
    pub(super) origin: String,
    pub(super) token: SecretString,
    pub(super) seen: Arc<Mutex<Vec<String>>>,
    task: JoinHandle<()>,
}
struct Data {
    origin: String,
    token: SecretString,
    seen: Arc<Mutex<Vec<String>>>,
}
impl Server {
    pub(super) async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}/", listener.local_addr().unwrap());
        let token: SecretString = Uuid::new_v4().to_string().into();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let state = Arc::new(Data {
            origin: origin.clone(),
            token: token.clone(),
            seen: seen.clone(),
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
            .collect::<std::collections::HashMap<_, _>>();
        assert!(fields.contains_key("client_secret"));
        assert!(
            fields.contains_key("code_verifier")
                || fields
                    .get("grant_type")
                    .is_some_and(|value| value == "refresh_token")
        );
        return Json(json!({"access_token":data.token.expose_secret(),"expires_in":28800,"refresh_token":Uuid::new_v4().to_string(),"refresh_token_expires_in":15897600})).into_response();
    }
    if path == "/user" {
        return Json(
            json!({"id":7,"login":"person-7","avatar_url":"https://github.example/avatar"}),
        )
        .into_response();
    }
    let url = Url::parse(&format!("http://fake{uri}")).unwrap();
    let page = url
        .query_pairs()
        .find(|(key, _)| key == "page")
        .map(|(_, value)| value.into_owned())
        .unwrap_or_else(|| "1".to_owned());
    if path == "/user/memberships/orgs" {
        assert!(headers.get("Authorization").is_some_and(|value| {
            value
                .to_str()
                .unwrap()
                .ends_with(data.token.expose_secret())
        }));
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
    if path.starts_with("/app/installations/70") && method == Method::GET {
        let (id, kind) = match path {
            "/app/installations/700" => (7, "User"),
            "/app/installations/701" => (8, "User"),
            _ => (7, "Organization"),
        };
        return Json(json!({"id":path.rsplit('/').next().unwrap().parse::<i64>().unwrap(),"account":{"id":id,"login":"account","type":kind}}))
            .into_response();
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
