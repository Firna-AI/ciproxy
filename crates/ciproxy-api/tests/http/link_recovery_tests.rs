//! Request-only waiting, original-person recovery, and direct-install preservation.

use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::dashboard::records::Role;
use url::Url;

use super::fixture_io::{CallbackOutcome, json};
use super::link_support::{begin, claims, event, memberships, pending, returned, scenario};
use super::setup_support::{browser, configure, finish};

#[tokio::test]
async fn request_return_and_browserless_approval_wait_for_original_person_and_current_admin_access()
{
    let (mut fixture, server, admin, added) = scenario().await;
    let state = begin(&fixture, &admin).await;
    let start = fixture
        .request(
            None,
            "GET",
            &format!("/github/setup?setup_action=request&state={state}"),
            None,
        )
        .await;
    let response = finish(&fixture, start, CallbackOutcome::Accepted).await;
    let waiting = browser(&response);
    let session = json(
        fixture
            .request(Some(&waiting), "GET", "/session", None)
            .await,
    )
    .await;
    assert_eq!(session["setup"]["kind"], "waiting_approval");
    assert!(
        fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .is_none()
    );
    let id = pending(
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap(),
    );
    assert!(
        fixture
            .database
            .store
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
    configure(&mut fixture, &server);
    memberships(&server, &[106]);
    let recovery = fixture.login(7, None).await;
    let links = claims(&fixture, &recovery).await;
    assert_eq!(links[0]["id"], id.to_string());
    assert_eq!(links[0]["state"], "waiting_for_admin");
    assert!(links[0]["workspace_id"].is_null());
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
            .organization_by_installation_id(1066)
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn lost_add_return_recovers_without_state_and_direct_callback_before_event_keeps_new_workspace_rule()
 {
    let (fixture, _server, admin, added) = scenario().await;
    let _state = begin(&fixture, &admin).await;
    let id = pending(
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap(),
    );
    let recovery = fixture.login(7, Some(Role::Admin)).await;
    let linked = fixture
        .database
        .store
        .installation_claim(id)
        .unwrap()
        .unwrap();
    assert_eq!(linked.workspace_id, Some(fixture.workspace));
    assert!(!linked.direct);
    assert_eq!(
        claims(&fixture, &recovery).await[0]["state"],
        "linked_inactive"
    );

    let (fixture, _server, admin, added) = scenario().await;
    let session = json(fixture.request(Some(&admin), "GET", "/session", None).await).await;
    let url = Url::parse(session["install_url"].as_str().unwrap()).unwrap();
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .unwrap()
        .1
        .into_owned();
    returned(&fixture, &state, CallbackOutcome::Accepted)
        .await
        .unwrap();
    let provisional = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    assert!(provisional.direct && !provisional.event_seen);
    assert!(provisional.organization_id.is_none());
    let outcome = fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    let saved = match outcome {
        InstallationOutcome::Organization(saved) => saved,
        other => panic!("direct install must register: {other:?}"),
    };
    assert_ne!(saved.workspace_id, fixture.workspace);
    let completed = fixture
        .database
        .store
        .installation_claim(provisional.id)
        .unwrap()
        .unwrap();
    assert!(completed.event_seen);
    assert_eq!(completed.workspace_id, Some(saved.workspace_id));
    assert_eq!(completed.organization_id, Some(saved.id));
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(saved.workspace_id, 7)
            .unwrap(),
        Some(Role::Admin)
    );
    assert!(
        fixture
            .database
            .store
            .organization_runnable(saved.workspace_id, saved.id)
            .unwrap()
    );
}
