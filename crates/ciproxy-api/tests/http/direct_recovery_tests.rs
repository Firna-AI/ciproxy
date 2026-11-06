//! Signed Direct creation survives preparation failure without Add proof.

use std::sync::atomic::Ordering;

use ciproxy_github::app::InstallationOutcome;
use ciproxy_github::app::webhook::InstallationAction;
use ciproxy_github::error::Error;
use ciproxy_store_interface::dashboard::records::Role;
use ciproxy_store_interface::organizations::AppState;
use uuid::Uuid;

use super::link_support::{claims, event, maintenance, scenario};

#[derive(Clone, Copy, Debug)]
enum Recovery {
    CreatedReplay,
    ExactClaim,
    Maintenance,
    BrowserRefresh,
}

#[tokio::test]
async fn failed_direct_created_recovers_without_add_proof_or_duplicate_registration() {
    for recovery in [
        Recovery::CreatedReplay,
        Recovery::ExactClaim,
        Recovery::Maintenance,
        Recovery::BrowserRefresh,
    ] {
        let (fixture, server, admin, added) = scenario().await;
        fixture.remote.fail_ensure.store(true, Ordering::SeqCst);
        let failed = fixture
            .app
            .installation(event(&added, 7, InstallationAction::Created))
            .await;
        assert!(matches!(failed, Err(Error::AccessUnavailable(503))));
        let claim = fixture
            .database
            .store
            .claim_for_installation(1066)
            .unwrap()
            .unwrap();
        assert_eq!(
            (claim.event_seen, claim.direct, claim.user_verified),
            (true, true, false)
        );
        assert_eq!(claim.organization_id, None);
        assert_eq!(claim.workspace_id, None);
        assert_eq!(
            fixture
                .database
                .store
                .organizations_by_github_id(106)
                .unwrap(),
            Vec::new()
        );
        let recovered = match recovery {
            Recovery::CreatedReplay => Some(
                fixture
                    .app
                    .installation(event(&added, 7, InstallationAction::Created))
                    .await
                    .unwrap(),
            ),
            Recovery::ExactClaim => Some(
                fixture
                    .app
                    .resolve_installation_link(claim.id)
                    .await
                    .unwrap(),
            ),
            Recovery::Maintenance => {
                maintenance(&fixture).await;
                None
            }
            Recovery::BrowserRefresh => {
                server
                    .control
                    .hide_added_installation
                    .store(true, Ordering::SeqCst);
                claims(&fixture, &admin).await;
                None
            }
        };
        let actual = match recovered {
            Some(InstallationOutcome::Organization(organization)) => Some(organization),
            Some(_) => None,
            None => fixture
                .database
                .store
                .organization_by_installation_id(1066)
                .unwrap(),
        };
        assert_eq!(
            actual.as_ref().map(|organization| (
                organization.github_id,
                organization.installation_id,
                organization.app_state
            )),
            Some((106, 1066, AppState::Active)),
            "failed Direct {recovery:?} must complete without Add proof"
        );
        let saved = actual.unwrap();
        assert_ne!(saved.workspace_id, fixture.workspace);
        assert!(
            fixture
                .database
                .store
                .organization_runnable(saved.workspace_id, saved.id)
                .unwrap()
        );
        assert_eq!(
            fixture
                .database
                .store
                .scale_sets(saved.workspace_id, saved.id)
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_role(saved.workspace_id, 7)
                .unwrap(),
            Some(Role::Admin)
        );
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_role(saved.workspace_id, 8)
                .unwrap(),
            None
        );
        assert_eq!(
            fixture
                .database
                .store
                .pending_installation_choices()
                .unwrap(),
            Vec::<Uuid>::new()
        );
        let completed = fixture
            .database
            .store
            .installation_claim(claim.id)
            .unwrap()
            .unwrap();
        assert_eq!(completed.organization_id, Some(saved.id));
        assert_eq!(completed.workspace_id, Some(saved.workspace_id));
        assert!(!completed.user_verified);
        assert_eq!(
            fixture
                .database
                .store
                .installation_workspace_offers(claim.id, 7)
                .unwrap(),
            Vec::<Uuid>::new()
        );
        let links = claims(&fixture, &admin).await;
        assert_eq!(links[0]["state"], "linked");
        assert_eq!(links[0]["reauthorization_required"], false);
        assert_eq!(
            fixture
                .app
                .installation(event(&added, 7, InstallationAction::Created))
                .await
                .unwrap(),
            InstallationOutcome::Organization(saved.clone())
        );
        assert_eq!(
            fixture
                .database
                .store
                .organizations_by_github_id(106)
                .unwrap(),
            vec![saved.clone()]
        );
        assert_eq!(
            fixture
                .database
                .store
                .dashboard_members(saved.workspace_id)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            fixture
                .database
                .store
                .pending_initial_admin(saved.workspace_id)
                .unwrap(),
            None
        );
    }
}
