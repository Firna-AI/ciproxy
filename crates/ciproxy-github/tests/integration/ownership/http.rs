//! Stateful scale sets and retained registrations behind the real SDK.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use serde_json::{Value, json};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[derive(Clone, Debug)]
pub(super) struct RemoteSet {
    pub(super) id: i64,
    pub(super) name: String,
    pub(super) group: i64,
    pub(super) labels: Vec<String>,
}

impl RemoteSet {
    fn wire(&self) -> Value {
        json!({"id":self.id,"name":self.name,"runnerGroupId":self.group,
            "labels":self.labels.iter().map(|name| json!({"name":name,"type":"System"})).collect::<Vec<_>>()})
    }
}

pub(super) struct RemoteState {
    pub(super) sets: BTreeMap<i64, RemoteSet>,
    pub(super) runners: BTreeMap<i64, i64>,
    pub(super) fail_create: Option<String>,
    pub(super) creates: Vec<i64>,
    pub(super) deletes: Vec<i64>,
    next_set: i64,
    next_runner: i64,
}

pub(super) struct Server {
    pub(super) origin: String,
    pub(super) state: Arc<Mutex<RemoteState>>,
    task: JoinHandle<()>,
}

impl Server {
    pub(super) async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("http://{}", listener.local_addr().unwrap());
        let state = Arc::new(Mutex::new(RemoteState {
            sets: BTreeMap::from([(
                53,
                RemoteSet {
                    id: 53,
                    name: "ciproxy".to_owned(),
                    group: 47,
                    labels: vec!["ciproxy".to_owned()],
                },
            )]),
            runners: BTreeMap::new(),
            fail_create: None,
            creates: vec![],
            deletes: vec![],
            next_set: 83,
            next_runner: 71,
        }));
        let router = Router::new()
            .fallback(handle)
            .with_state((origin.clone(), state.clone()));
        let task = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
        Self {
            origin,
            state,
            task,
        }
    }

    pub(super) fn eligible_runners(&self, label: &str) -> Vec<i64> {
        let state = self.state.lock().unwrap();
        state
            .runners
            .iter()
            .filter(|(_, set_id)| {
                state.sets[*set_id]
                    .labels
                    .iter()
                    .any(|value| value == label)
            })
            .map(|(id, _)| *id)
            .collect()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn handle(
    State((origin, state)): State<(String, Arc<Mutex<RemoteState>>)>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    bytes: Bytes,
) -> Response {
    let path = uri.path();
    if path.ends_with("/access_tokens") {
        if !headers
            .get("Authorization")
            .is_some_and(|value| value == "Bearer test-app-jwt")
        {
            return StatusCode::UNAUTHORIZED.into_response();
        }
        return (
            StatusCode::CREATED,
            Json(json!({"token":"test-installation-token","expires_at":"2099-01-01T00:00:00Z"})),
        )
            .into_response();
    }
    if path.ends_with("/registration-token") {
        return (
            StatusCode::CREATED,
            Json(json!({"token":"test-registration-token"})),
        )
            .into_response();
    }
    if path.ends_with("/runner-registration") {
        return Json(
            json!({"url":format!("{origin}/actions"),"token":"e30.eyJleHAiOjQxMDI0NDQ4MDB9.test"}),
        )
        .into_response();
    }
    let body: Value = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    let mut state = state.lock().unwrap();
    if let Some(id) = path.split("/agents/").nth(1)
        && method == Method::DELETE
    {
        return if state.runners.remove(&id.parse().unwrap()).is_some() {
            StatusCode::NO_CONTENT
        } else {
            StatusCode::NOT_FOUND
        }
        .into_response();
    }
    if let Some(suffix) = path.split("/runnerscalesets/").nth(1) {
        let mut parts = suffix.split('/');
        let id = parts.next().unwrap().parse::<i64>().unwrap();
        if parts.next() == Some("generatejitconfig") && method == Method::POST {
            if !state.sets.contains_key(&id) {
                return StatusCode::NOT_FOUND.into_response();
            }
            let runner_id = state.next_runner;
            state.next_runner += 1;
            state.runners.insert(runner_id, id);
            return Json(json!({"runner":{"id":runner_id,"name":body["name"],"runnerScaleSetId":id},"encodedJITConfig":"test-one-use-configuration"})).into_response();
        }
        if !state.sets.contains_key(&id) {
            return StatusCode::NOT_FOUND.into_response();
        }
        if method == Method::DELETE {
            state.deletes.push(id);
            state.sets.remove(&id);
            return StatusCode::NO_CONTENT.into_response();
        }
        if method == Method::PATCH {
            let name = body["name"].as_str().unwrap();
            let group = state.sets[&id].group;
            if state
                .sets
                .values()
                .any(|set| set.id != id && set.group == group && set.name == name)
            {
                return StatusCode::CONFLICT.into_response();
            }
            let set = state.sets.get_mut(&id).unwrap();
            set.name = name.to_owned();
            if let Some(labels) = body["labels"].as_array() {
                set.labels = labels
                    .iter()
                    .map(|label| label["name"].as_str().unwrap().to_owned())
                    .collect();
            }
        }
        return Json(state.sets[&id].wire()).into_response();
    }
    if path.ends_with("/runnerscalesets") && method == Method::GET {
        let query = url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
            .collect::<BTreeMap<_, _>>();
        let name: Option<&str> = query.get("name").map(|value| value.as_ref());
        let group = query
            .get("runnerGroupId")
            .map(|value| value.parse::<i64>().unwrap());
        let sets = state
            .sets
            .values()
            .filter(|set| {
                name.is_none_or(|name| set.name == name)
                    && group.is_none_or(|group| set.group == group)
            })
            .map(RemoteSet::wire)
            .collect::<Vec<_>>();
        return Json(json!({"count":sets.len(),"value":sets})).into_response();
    }
    if path.ends_with("/runnerscalesets") && method == Method::POST {
        let name = body["name"].as_str().unwrap();
        let group = body["runnerGroupId"].as_i64().unwrap();
        if state.fail_create.as_deref() == Some(name) {
            state.fail_create = None;
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        }
        if state
            .sets
            .values()
            .any(|set| set.group == group && set.name == name)
        {
            return StatusCode::CONFLICT.into_response();
        }
        let id = state.next_set;
        state.next_set += 1;
        let set = RemoteSet {
            id,
            name: name.to_owned(),
            group,
            labels: body["labels"]
                .as_array()
                .unwrap()
                .iter()
                .map(|label| label["name"].as_str().unwrap().to_owned())
                .collect(),
        };
        let wire = set.wire();
        state.sets.insert(id, set);
        state.creates.push(id);
        return Json(wire).into_response();
    }
    StatusCode::NOT_FOUND.into_response()
}
