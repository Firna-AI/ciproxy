//! A personal-account return preserves Add mode and retains no person or grant.

use std::sync::atomic::Ordering;

use ciproxy_store_interface::dashboard::records::Role;
use sha2::{Digest, Sha256};
use url::Url;

use super::adapter_server::Server;
use super::fixture::Fixture;
use super::fixture_io::{CallbackOutcome, json};
use super::link_support;
use super::setup_support::{browser, configure, finish};

#[tokio::test]
async fn fresh_personal_setup_records_only_verified_token_free_metadata() {
    let server = Server::new().await;
    let mut fixture = Fixture::new();
    configure(&mut fixture, &server);
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let state = link_support::begin(&fixture, &admin).await;
    server.control.memberships.store(false, Ordering::SeqCst);
    let response = finish(
        &fixture,
        fixture
            .request(
                None,
                "GET",
                &format!("/github/setup?installation_id=700&state={state}"),
                None,
            )
            .await,
        CallbackOutcome::Accepted,
    )
    .await;
    let signed = browser(&response);
    assert!(
        !fixture.database.store.has_pending_add(7).unwrap(),
        "a verified personal return resolves that entry, while its retry keeps Add mode"
    );
    let session = json(
        fixture
            .request(Some(&signed), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["setup"]["kind"], "organizations_only");
    let retry = Url::parse(session["install_url"].as_str().unwrap()).unwrap();
    let retry_state = retry
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    let attempt = fixture
        .database
        .store
        .installation_attempt(&Sha256::digest(retry_state.as_bytes()))
        .unwrap()
        .unwrap();
    assert!(
        !attempt.direct,
        "a personal-account Add retry must stay Add"
    );
    assert!(
        fixture
            .database
            .store
            .installation_claims(7)
            .unwrap()
            .is_empty()
    );
    assert_eq!(session["setup"]["installation_id"], "700");
    assert_eq!(
        session["setup"]["remove_url"],
        "https://github.com/settings/installations/700"
    );
    assert_eq!(session["workspaces"].as_array().unwrap().len(), 0);
    assert!(
        fixture
            .database
            .store
            .dashboard_user_by_github(7)
            .unwrap()
            .is_none()
    );
    assert!(
        fixture
            .database
            .store
            .organizations_by_github_id(7)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        session["pending_install_requests"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
