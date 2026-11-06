//! Controlled configured-App and user-visible installation facts.

use std::sync::atomic::{AtomicBool, Ordering};

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub(super) struct Control {
    pub(super) memberships: AtomicBool,
    pub(super) fail_user_installations: AtomicBool,
    pub(super) fail_app_installations: AtomicBool,
}
impl Control {
    pub(super) fn new() -> Self {
        Self {
            memberships: AtomicBool::new(true),
            fail_user_installations: AtomicBool::new(false),
            fail_app_installations: AtomicBool::new(false),
        }
    }
}

pub(super) fn respond(control: &Control, path: &str, page: &str) -> Option<Response> {
    if path == "/user/installations" {
        if control.fail_user_installations.load(Ordering::SeqCst) {
            return Some(StatusCode::SERVICE_UNAVAILABLE.into_response());
        }
        let installations = if page == "1" {
            (10000..10100).map(|id|json!({"id":id,"account":{"id":id+20000,"login":"other-access","type":"Organization"}})).collect::<Vec<_>>()
        } else {
            vec![
                json!({"id":101,"account":{"id":100,"login":"customer","type":"Organization"}}),
                json!({"id":1066,"account":{"id":106,"login":"not-registered","type":"Organization"}}),
                json!({"id":700,"account":{"id":7,"login":"person-7","type":"User"}}),
                json!({"id":702,"account":{"id":7,"login":"not-member","type":"Organization"}}),
                json!({"id":703,"account":{"id":8,"login":"person-8","type":"User"}}),
                json!({"id":704,"account":{"id":7,"login":"wrong-kind","type":"User"}}),
                json!({"id":705,"account":{"id":200,"login":"wrong-account","type":"Organization"}}),
            ]
        };
        return Some(Json(json!({"installations":installations})).into_response());
    }
    if let Some(id) = path
        .strip_prefix("/app/installations/")
        .and_then(|value| value.parse::<i64>().ok())
    {
        if control.fail_app_installations.load(Ordering::SeqCst) {
            return Some(StatusCode::SERVICE_UNAVAILABLE.into_response());
        }
        let (account, login, kind) = match id {
            101 => (100, "customer", "Organization"),
            1066 => (106, "not-registered", "Organization"),
            201 => (200, "invisible", "Organization"),
            700 => (7, "person-7", "User"),
            701 | 703 => (8, "person-8", "User"),
            702 => (7, "not-member", "Organization"),
            704 | 705 => (100, "customer", "Organization"),
            _ => return Some(StatusCode::NOT_FOUND.into_response()),
        };
        return Some(Json(json!({"id":id,"account":{"id":account,"login":login,"type":kind},"suspended_at":null})).into_response());
    }
    None
}
