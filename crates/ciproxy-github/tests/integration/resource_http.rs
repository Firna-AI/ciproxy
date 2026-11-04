//! Known GitHub resource and installation responses for recovery integration.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU16, AtomicUsize, Ordering};

use axum::Json;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

pub(super) struct ResourceHttp {
    pub(super) group_exists: AtomicBool,
    pub(super) group_name: Mutex<String>,
    pub(super) group_creates: AtomicUsize,
    pub(super) set_id: AtomicI64,
    pub(super) set_group: AtomicI64,
    pub(super) missing_status: AtomicU16,
    pub(super) installation_hidden: AtomicBool,
    pub(super) suspended: AtomicBool,
    pub(super) app_jwt_reads: AtomicUsize,
    pub(super) organization_login: Mutex<String>,
    pub(super) account_id: AtomicI64,
    pub(super) auth_failure: Mutex<Option<String>>,
}

impl ResourceHttp {
    pub(super) fn new() -> Self {
        Self {
            group_exists: AtomicBool::new(false),
            group_name: Mutex::new("owner-renamed".to_owned()),
            group_creates: AtomicUsize::new(0),
            set_id: AtomicI64::new(53),
            set_group: AtomicI64::new(47),
            missing_status: AtomicU16::new(0),
            installation_hidden: AtomicBool::new(false),
            suspended: AtomicBool::new(false),
            app_jwt_reads: AtomicUsize::new(0),
            organization_login: Mutex::new("customer".to_owned()),
            account_id: AtomicI64::new(17),
            auth_failure: Mutex::new(None),
        }
    }
}

pub(super) fn respond(
    state: &ResourceHttp,
    method: &Method,
    path: &str,
    body: &Value,
    app_jwt: bool,
) -> Option<Response> {
    if state
        .auth_failure
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|route| path.ends_with(route))
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }
    let login = state.organization_login.lock().unwrap().clone();
    if path.ends_with(&format!("/orgs/{login}/installation"))
        || path.ends_with("/app/installations/19")
    {
        if !app_jwt {
            return Some(StatusCode::UNAUTHORIZED.into_response());
        }
        state.app_jwt_reads.fetch_add(1, Ordering::SeqCst);
        if state.installation_hidden.load(Ordering::SeqCst) {
            return Some(StatusCode::NOT_FOUND.into_response());
        }
        let suspended = state
            .suspended
            .load(Ordering::SeqCst)
            .then_some("2026-11-04T12:00:00Z");
        return Some(Json(json!({"id":19,"account":{"id":state.account_id.load(Ordering::SeqCst),"login":login,"type":"Organization"},"suspended_at":suspended})).into_response());
    }
    if path.ends_with("/runner-groups")
        && *method == Method::GET
        && state.group_exists.load(Ordering::SeqCst)
    {
        return Some(Json(json!({"runner_groups":[{"id":47,"name":*state.group_name.lock().unwrap(),"inherited":false}]})).into_response());
    }
    if path.ends_with("/runner-groups/47") && *method == Method::GET {
        return Some(if state.group_exists.load(Ordering::SeqCst) {
            Json(json!({"id":47,"name":*state.group_name.lock().unwrap(),"inherited":false}))
                .into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        });
    }
    if path.ends_with("/runner-groups/47") && *method == Method::PATCH {
        *state.group_name.lock().unwrap() = body["name"].as_str().unwrap().to_owned();
        return Some(Json(json!({"id":47})).into_response());
    }
    if path.ends_with("/runner-groups") && *method == Method::POST {
        let existed = state.group_exists.swap(true, Ordering::SeqCst);
        state.group_creates.fetch_add(1, Ordering::SeqCst);
        *state.group_name.lock().unwrap() = body["name"].as_str().unwrap().to_owned();
        return Some((StatusCode::CREATED, Json(json!({"id":if existed {97} else {47},"name":*state.group_name.lock().unwrap(),"inherited":false}))).into_response());
    }
    let missing = state.missing_status.load(Ordering::SeqCst);
    if missing != 0
        && ((*method == Method::POST
            && (path.ends_with("/sessions") || path.ends_with("/generatejitconfig")))
            || path == "/queue")
    {
        return Some(StatusCode::from_u16(missing).unwrap().into_response());
    }
    if path.ends_with("/runnerscalesets/53") && *method == Method::GET {
        return Some(if state.set_id.load(Ordering::SeqCst) == 53 {
            Json(json!({"id":53,"name":"ciproxy","runnerGroupId":state.set_group.load(Ordering::SeqCst)})).into_response()
        } else {
            StatusCode::NO_CONTENT.into_response()
        });
    }
    if path.ends_with("/runnerscalesets") && *method == Method::GET {
        let id = state.set_id.load(Ordering::SeqCst);
        let value = if id == 0 {
            vec![]
        } else {
            vec![
                json!({"id":id,"name":"ciproxy","runnerGroupId":state.set_group.load(Ordering::SeqCst)}),
            ]
        };
        return Some(Json(json!({"count":value.len(),"value":value})).into_response());
    }
    if path.ends_with("/runnerscalesets") && *method == Method::POST {
        state.set_id.store(83, Ordering::SeqCst);
        state
            .set_group
            .store(body["runnerGroupId"].as_i64().unwrap(), Ordering::SeqCst);
        return Some(Json(json!({"id":83,"name":"ciproxy","runnerGroupId":state.set_group.load(Ordering::SeqCst)})).into_response());
    }
    None
}
