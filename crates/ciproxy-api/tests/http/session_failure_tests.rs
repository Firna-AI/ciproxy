//! Recoverable cookie failures must leave the previous private sign-in usable.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use axum::http::StatusCode;
use ciproxy_api::access::AccessService;
use ciproxy_api::crypto::{AeadCredentialCipher, CredentialCipher};
use ciproxy_api::error::{Error, Result};
use ciproxy_store_interface::dashboard::records::Role;
use secrecy::{ExposeSecret, SecretString};
use uuid::Uuid;

use super::fixture::Fixture;
use super::fixture_io::CallbackOutcome;
use super::setup_support::finish;

struct FailingBrowserCipher {
    fail: AtomicBool,
    inner: AeadCredentialCipher,
}

#[async_trait]
impl CredentialCipher for FailingBrowserCipher {
    async fn seal(&self, id: Uuid, value: SecretString) -> Result<Vec<u8>> {
        if self.fail.load(Ordering::SeqCst) && value.expose_secret().contains("\"session_id\"") {
            return Err(Error::Unavailable);
        }
        self.inner.seal(id, value).await
    }

    async fn open(&self, id: Uuid, value: Vec<u8>) -> Result<SecretString> {
        self.inner.open(id, value).await
    }
}

#[tokio::test]
async fn cookie_sealing_failure_keeps_the_existing_private_sign_in() {
    let mut fixture = Fixture::new();
    let current = fixture.login(7, Some(Role::Viewer)).await;
    let cipher = Arc::new(FailingBrowserCipher {
        fail: AtomicBool::new(false),
        inner: AeadCredentialCipher::new(&[17; 32]),
    });
    fixture.state.access = Arc::new(AccessService::new(
        fixture.database.store.clone(),
        fixture.identity.clone(),
        cipher.clone(),
        fixture.runtime.clone(),
    ));
    let begin = fixture
        .request(Some(&current), "GET", "/auth/github", None)
        .await;
    cipher.fail.store(true, Ordering::SeqCst);
    finish(&fixture, begin, CallbackOutcome::Refused).await;
    assert_eq!(
        fixture
            .request(Some(&current), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::OK
    );
}
