//! Failed authorization preserves first install results for the successful retry.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use axum::http::StatusCode;
use ciproxy_api::crypto::{AeadCredentialCipher, CredentialCipher};
use ciproxy_api::error::{Error, Result};
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use secrecy::{ExposeSecret, SecretString};
use serde_json::{Value, json};
use uuid::Uuid;

use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, event, memberships, scenario};
use super::setup_support::{browser, configure_with_cipher, finish};

struct BrowserCipher {
    fail: AtomicBool,
    inner: AeadCredentialCipher,
}

#[async_trait]
impl CredentialCipher for BrowserCipher {
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

async fn authorize(
    fixture: &Fixture,
    prior: &Browser,
    outcome: CallbackOutcome,
) -> Option<Browser> {
    let response = finish(
        fixture,
        fixture
            .request(Some(prior), "GET", "/auth/github", None)
            .await,
        outcome,
    )
    .await;
    match outcome {
        CallbackOutcome::Accepted => Some(browser(&response)),
        CallbackOutcome::Refused => None,
    }
}

async fn session(fixture: &Fixture, browser: &Browser) -> Value {
    let response = fixture
        .request(Some(browser), "GET", "/session", None)
        .await;
    assert_eq!(response.status(), StatusCode::OK);
    json(response).await
}

fn seed_claim(fixture: &Fixture, direct: bool) {
    fixture
        .database
        .store
        .save_installation_claim(
            &InstallationClaim {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap(),
                installation_id: 1066,
                github_id: 106,
                login: "not-registered".to_owned(),
                requester_id: 7,
                event_seen: false,
                user_verified: false,
                direct,
                workspace_id: None,
                organization_id: None,
                verified_at: None,
            },
            None,
            InstallationResultDelivery::Untracked,
        )
        .unwrap();
}

async fn assert_failed_authorization_keeps_business_records(fixture: &Fixture, prior: &Browser) {
    assert_eq!(
        fixture
            .request(Some(prior), "GET", &fixture.path("/machines"), None)
            .await
            .status(),
        StatusCode::OK
    );
    let saved = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    let organization = fixture
        .database
        .store
        .organization_by_installation_id(1066)
        .unwrap()
        .unwrap();
    assert_eq!(saved.id.to_string(), "00000000-0000-0000-0000-000000000011");
    assert_eq!(
        (saved.requester_id, saved.event_seen, saved.user_verified),
        (7, true, true)
    );
    assert_eq!(saved.organization_id, Some(organization.id));
    assert_eq!(saved.workspace_id, Some(organization.workspace_id));
    assert_eq!(
        session(fixture, prior).await["installation_continuation"],
        json!([])
    );
}

async fn assert_successful_retry_delivers_once(fixture: &Fixture, prior: &Browser) {
    let current = authorize(fixture, prior, CallbackOutcome::Accepted)
        .await
        .unwrap();
    let resolved = session(fixture, &current).await;
    assert_eq!(
        resolved["installation_continuation"],
        json!(["00000000-0000-0000-0000-000000000011"])
    );
    assert_eq!(
        session(fixture, &current).await["installation_continuation"],
        resolved["installation_continuation"]
    );
    let repeated = authorize(fixture, &current, CallbackOutcome::Accepted)
        .await
        .unwrap();
    assert_eq!(
        session(fixture, &repeated).await["installation_continuation"],
        json!([])
    );
}

#[tokio::test]
async fn cookie_preparation_failure_preserves_direct_result_for_successful_retry() {
    let (mut fixture, server, prior, added) = scenario().await;
    memberships(&server, &[100, 106]);
    let cipher = Arc::new(BrowserCipher {
        fail: AtomicBool::new(false),
        inner: AeadCredentialCipher::new(&[17; 32]),
    });
    configure_with_cipher(&mut fixture, &server, cipher.clone());
    seed_claim(&fixture, true);
    fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    let completed = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    assert!(!completed.user_verified);
    assert!(completed.organization_id.is_some());
    let begin = fixture
        .request(Some(&prior), "GET", "/auth/github", None)
        .await;
    cipher.fail.store(true, Ordering::SeqCst);
    finish(&fixture, begin, CallbackOutcome::Refused).await;
    cipher.fail.store(false, Ordering::SeqCst);
    assert_failed_authorization_keeps_business_records(&fixture, &prior).await;
    assert_successful_retry_delivers_once(&fixture, &prior).await;
}

#[tokio::test]
async fn continuation_save_failure_preserves_direct_and_add_results_for_successful_retry() {
    for direct in [true, false] {
        let (fixture, server, prior, added) = scenario().await;
        memberships(&server, &[100, 106]);
        if !direct {
            begin(&fixture, &prior).await;
        }
        seed_claim(&fixture, direct);
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap();
        let pending = fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .unwrap();
        assert_eq!(pending.organization_id.is_some(), direct);
        assert!(!pending.user_verified);
        let mut connection = PgConnection::establish(&fixture.database.url).unwrap();
        connection
            .batch_execute(
                "CREATE FUNCTION fail_continuation() RETURNS trigger AS $$
         BEGIN RAISE EXCEPTION 'controlled continuation write failure'; END
         $$ LANGUAGE plpgsql;
         CREATE TRIGGER fail_continuation BEFORE INSERT OR UPDATE ON dashboard_continuations
         FOR EACH ROW EXECUTE FUNCTION fail_continuation();",
            )
            .unwrap();
        authorize(&fixture, &prior, CallbackOutcome::Refused).await;
        assert_failed_authorization_keeps_business_records(&fixture, &prior).await;
        let linked = fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .unwrap();
        assert_eq!(linked.direct, direct);
        assert_eq!(linked.workspace_id == Some(fixture.workspace), !direct);
        connection
            .batch_execute("DROP TRIGGER fail_continuation ON dashboard_continuations")
            .unwrap();
        assert_successful_retry_delivers_once(&fixture, &prior).await;
    }
}
