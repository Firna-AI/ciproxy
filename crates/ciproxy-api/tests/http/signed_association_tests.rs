//! A foreign provisional return cannot select the signed requester's flow.

use std::sync::atomic::Ordering;

use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use super::fixture_io::CallbackOutcome;
use super::link_support::{self, event, returned, scenario};

#[tokio::test]
async fn signed_direct_requester_keeps_new_workspace_after_another_members_return() {
    let (fixture, server, _admin, added) = scenario().await;
    server.control.person.store(8, Ordering::SeqCst);
    let other = fixture.login(8, Some(Role::Admin)).await;
    let state = link_support::start(&fixture, &other).await;
    returned(&fixture, &state, CallbackOutcome::Accepted)
        .await
        .unwrap();
    let provisional = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    assert_eq!(
        (
            provisional.requester_id,
            provisional.event_seen,
            provisional.direct,
            provisional.user_verified,
        ),
        (8, false, true, true)
    );
    assert_eq!(provisional.workspace_id, None);
    assert_eq!(
        (
            fixture.database.store.has_pending_add(7).unwrap(),
            fixture
                .database
                .store
                .has_add_installation(7, 1066)
                .unwrap(),
        ),
        (false, false)
    );
    server.control.person.store(7, Ordering::SeqCst);
    let outcome = fixture
        .app
        .installation(event(&added, 7, InstallationAction::Created))
        .await
        .unwrap();
    let organization = match outcome {
        InstallationOutcome::Organization(organization) => Some(organization),
        _ => None,
    };
    assert_eq!(
        organization.as_ref().map(|organization| (
            organization.github_id,
            organization.installation_id,
            organization.app_state,
        )),
        Some((106, 1066, AppState::Active)),
        "signed Direct creation must register after another member's provisional return"
    );
    let organization = organization.unwrap();
    assert_ne!(organization.workspace_id, fixture.workspace);
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(organization.workspace_id, 7)
            .unwrap(),
        Some(Role::Admin)
    );
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_role(organization.workspace_id, 8)
            .unwrap(),
        None
    );
    let signed = fixture
        .database
        .store
        .claim_for_installation(1066)
        .unwrap()
        .unwrap();
    assert_eq!(signed.id, provisional.id);
    assert_eq!(
        (
            signed.requester_id,
            signed.event_seen,
            signed.direct,
            signed.user_verified
        ),
        (7, true, true, false),
        "signed association must discard another member's proof"
    );
    assert_eq!(signed.workspace_id, Some(organization.workspace_id));
    assert_eq!(signed.organization_id, Some(organization.id));
    assert_eq!(signed.verified_at, None);
    assert_eq!(
        fixture
            .database
            .store
            .installation_workspace_offers(signed.id, 8)
            .unwrap(),
        Vec::<Uuid>::new()
    );
    server.control.person.store(8, Ordering::SeqCst);
    returned(&fixture, &state, CallbackOutcome::Refused).await;
    assert_eq!(
        fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .map(|claim| (
                claim.id,
                claim.requester_id,
                claim.direct,
                claim.user_verified
            )),
        Some((signed.id, 7, true, false))
    );
    assert_eq!(
        fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await
            .unwrap(),
        InstallationOutcome::Organization(organization.clone())
    );
    assert_eq!(
        fixture
            .database
            .store
            .organizations_by_github_id(106)
            .unwrap(),
        vec![organization.clone()]
    );
    assert_eq!(
        fixture
            .database
            .store
            .dashboard_members(organization.workspace_id)
            .unwrap()
            .len(),
        1
    );
}
