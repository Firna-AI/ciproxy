//! Direct setup proof records canonical claims before their proof state changes.

use std::collections::BTreeSet;

use ciproxy_api::runtime::Runtime;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use serde_json::{Value, json};
use url::Url;
use uuid::Uuid;

use super::fixture::{Browser, Fixture};
use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{event, memberships, scenario};
use super::setup_support::{browser, finish};

async fn session(fixture: &Fixture, browser: &Browser) -> Value {
    json(
        fixture
            .request(Some(browser), "GET", "/session", None)
            .await,
    )
    .await
}

async fn setup_return(fixture: &Fixture, prior: &Browser, state: &str) -> Browser {
    browser(
        &finish(
            fixture,
            fixture
                .request(
                    Some(prior),
                    "GET",
                    &format!("/github/setup?installation_id=1066&state={state}"),
                    None,
                )
                .await,
            CallbackOutcome::Accepted,
        )
        .await,
    )
}

#[tokio::test]
async fn completed_direct_setup_names_its_first_proof_and_every_other_result_without_history() {
    for mixed in [false, true] {
        let (fixture, server, prior, added) = scenario().await;
        memberships(&server, &[100, 106, 206]);
        let history = InstallationClaim {
            id: Uuid::new_v4(),
            installation_id: fixture.organization.installation_id,
            github_id: fixture.organization.github_id,
            login: fixture.organization.login.clone(),
            requester_id: 7,
            event_seen: true,
            user_verified: true,
            direct: true,
            workspace_id: Some(fixture.workspace),
            organization_id: Some(fixture.organization.id),
            verified_at: Some(fixture.runtime.now()),
        };
        fixture
            .database
            .store
            .save_installation_claim(&history, None, InstallationResultDelivery::Untracked)
            .unwrap();
        let initial = session(&fixture, &prior).await;
        let url = Url::parse(initial["install_url"].as_str().unwrap()).unwrap();
        let state = url
            .query_pairs()
            .find(|(key, _)| key == "state")
            .unwrap()
            .1
            .into_owned();
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap();
        let target = fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .unwrap();
        assert!(!target.user_verified);
        assert!(target.organization_id.is_some());
        let mut expected = BTreeSet::from([target.id]);
        if mixed {
            let mut other = added.clone();
            other.id = Uuid::new_v4();
            other.installation_id = 2066;
            other.github_id = 206;
            other.login = "other-first-proof".to_owned();
            fixture
                .remote
                .organizations
                .lock()
                .unwrap()
                .push(other.clone());
            server.control.extra_installations.lock().unwrap().push((
                other.installation_id,
                other.github_id,
                other.login.clone(),
            ));
            fixture
                .app
                .installation(event(&other, 7, InstallationAction::Created))
                .await
                .unwrap();
            let other = fixture
                .database
                .store
                .claim_for_installation(2066)
                .unwrap()
                .unwrap();
            expected.insert(other.id);
        }
        let current = setup_return(&fixture, &prior, &state).await;
        let resolved = session(&fixture, &current).await;
        let actual: BTreeSet<Uuid> =
            serde_json::from_value(resolved["installation_continuation"].clone()).unwrap();
        assert_eq!(actual, expected, "mixed={mixed}");
        assert!(!actual.contains(&history.id));
        assert_eq!(resolved["expires_at"], initial["expires_at"]);
        assert_eq!(
            session(&fixture, &current).await["installation_continuation"],
            resolved["installation_continuation"]
        );
        let saved = fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .unwrap();
        assert_eq!(saved.id, target.id);
        assert_eq!(saved.organization_id, target.organization_id);
        assert!(saved.user_verified);
        let repeated = setup_return(&fixture, &current, &state).await;
        assert_eq!(
            session(&fixture, &repeated).await["installation_continuation"],
            json!([])
        );
    }
}
