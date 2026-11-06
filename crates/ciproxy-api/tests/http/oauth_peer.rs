//! Rotating OAuth pairs and secret-safe exchange capture for the local peer.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use axum::Json;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use secrecy::{ExposeSecret, SecretString};
use serde_json::json;
use uuid::Uuid;

pub(super) struct OAuthPeer {
    pair: Mutex<Pair>,
    pub(super) verifiers: Mutex<Vec<SecretString>>,
    pub(super) fail_memberships: AtomicBool,
    pub(super) refreshes: AtomicUsize,
}

struct Pair {
    access: SecretString,
    refresh: SecretString,
}

impl OAuthPeer {
    pub(super) fn new(access: SecretString) -> Self {
        Self {
            pair: Mutex::new(Pair {
                access,
                refresh: Uuid::new_v4().to_string().into(),
            }),
            verifiers: Mutex::new(Vec::new()),
            fail_memberships: AtomicBool::new(false),
            refreshes: AtomicUsize::new(0),
        }
    }

    pub(super) fn token(&self, fields: &HashMap<String, String>) -> Response {
        let mut pair = self.pair.lock().unwrap();
        if fields
            .get("grant_type")
            .is_some_and(|value| value == "refresh_token")
        {
            if fields.get("refresh_token").map(String::as_str) != Some(pair.refresh.expose_secret())
            {
                return StatusCode::UNAUTHORIZED.into_response();
            }
            pair.access = Uuid::new_v4().to_string().into();
            pair.refresh = Uuid::new_v4().to_string().into();
            self.refreshes.fetch_add(1, Ordering::SeqCst);
        } else if let Some(verifier) = fields.get("code_verifier") {
            self.verifiers.lock().unwrap().push(verifier.clone().into());
        } else {
            return StatusCode::BAD_REQUEST.into_response();
        }
        Json(json!({
            "access_token": pair.access.expose_secret(),
            "expires_in": 28800,
            "refresh_token": pair.refresh.expose_secret(),
            "refresh_token_expires_in": 15897600
        }))
        .into_response()
    }

    pub(super) fn membership_error(&self, headers: &HeaderMap) -> Option<Response> {
        if self.fail_memberships.swap(false, Ordering::SeqCst) {
            return Some(StatusCode::SERVICE_UNAVAILABLE.into_response());
        }
        let pair = self.pair.lock().unwrap();
        let authorized = headers.get("Authorization").is_some_and(|header| {
            header
                .to_str()
                .ok()
                .and_then(|value| value.strip_prefix("Bearer "))
                == Some(pair.access.expose_secret())
        });
        (!authorized).then(|| StatusCode::UNAUTHORIZED.into_response())
    }
}
