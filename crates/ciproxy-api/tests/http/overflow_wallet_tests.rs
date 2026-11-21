//! Dashboard wallet selection crosses verified Boat HTTPS and real PostgreSQL.

use axum::http::StatusCode;
use ciproxy_store_interface::dashboard::records::Role;
use serde_json::{Value, json};

use crate::boat_facts::{identity, limits};
use crate::boat_peer::{Peer, step};

use super::fixture::Fixture;
use super::fixture_io::json;
use super::overflow_support::with_factory;

const ORGANIZATION: &str = "team_customer";

#[tokio::test]
async fn organization_key_checks_and_connects_its_own_wallet() {
    connects_wallet(
        json!([{"id":ORGANIZATION,"name":"Customer Organization","type":"org","active":true}]),
        json!([{"wallet":ORGANIZATION,"name":"Customer Organization"}]),
        ORGANIZATION,
    )
    .await;
}

#[tokio::test]
async fn personal_key_lists_all_wallets_and_connects_a_chosen_organization() {
    connects_wallet(
        json!([
            {"id":"user_customer","name":"Personal","type":"personal","active":true},
            {"id":ORGANIZATION,"name":"Customer Organization","type":"org","active":false},
            {"id":"team_other","name":"Other Organization","type":"org","active":false}
        ]),
        json!([
            {"wallet":"personal","name":"Customer Boat"},
            {"wallet":ORGANIZATION,"name":"Customer Organization"},
            {"wallet":"team_other","name":"Other Organization"}
        ]),
        "personal",
    )
    .await;
}

async fn connects_wallet(orgs: Value, expected_wallets: Value, selected_wallet: &str) {
    let listing = json!({"ok":true,"type":"org.list","activeWalletChosen":true,"orgs":orgs});
    let mut organization_limits = limits(true);
    organization_limits["teamId"] = json!(ORGANIZATION);
    let peer = Peer::new(vec![
        step("GET", "/api/v1/me", 200, identity()).wallet(None),
        step("GET", "/api/v1/orgs", 200, listing.clone()).wallet(None),
        step("GET", "/api/v1/me", 200, identity()).wallet(Some(ORGANIZATION)),
        step("GET", "/api/v1/limits", 200, organization_limits.clone()).wallet(Some(ORGANIZATION)),
        step("GET", "/api/v1/orgs", 200, listing).wallet(Some(ORGANIZATION)),
        step("GET", "/api/v1/limits", 200, organization_limits).wallet(Some(ORGANIZATION)),
    ])
    .await;
    let mut fixture = Fixture::new();
    with_factory(&mut fixture, peer.factory.clone());
    let admin = fixture.login(7, Some(Role::Admin)).await;
    let key = json!({"api_key":"fixture-key"});
    let checked = fixture
        .request(
            Some(&admin),
            "POST",
            &fixture.path("/overflow/wallets"),
            Some(key),
        )
        .await;
    assert_eq!(checked.status(), StatusCode::OK);
    let checked = json(checked).await;
    assert_eq!(checked["wallets"], expected_wallets);
    assert_eq!(checked["selected_wallet"], selected_wallet);
    assert!(!checked.to_string().contains("fixture-key"));
    assert_eq!(
        fixture
            .database
            .store
            .boat_accounts(fixture.workspace)
            .unwrap(),
        Vec::new()
    );
    assert_eq!(
        fixture
            .request(
                Some(&admin),
                "POST",
                &fixture.path("/overflow/connection"),
                Some(json!({
                    "api_key":"fixture-key",
                    "wallet":ORGANIZATION,
                    "spend_limit_cents":"20000"
                })),
            )
            .await
            .status(),
        StatusCode::OK
    );
    let saved = fixture
        .database
        .store
        .boat_runtime_snapshot(fixture.workspace)
        .unwrap();
    assert_eq!(saved.accounts.len(), 1);
    assert_eq!(saved.accounts[0].state.wallet, ORGANIZATION);
    assert_eq!(
        saved.accounts[0].account.account_name,
        "Customer Organization"
    );
    assert_eq!(saved.spend_limit_usd_micros, Some(200_000_000));
    let displayed = json(
        fixture
            .request(Some(&admin), "GET", &fixture.path("/overflow"), None)
            .await,
    )
    .await;
    assert_eq!(displayed["account"]["name"], "Customer Organization");
    peer.assert_consumed();
    let captured = peer.captured();
    assert_eq!(captured.len(), 6);
    assert!(captured.iter().all(|request| request.body.is_null()));
}
