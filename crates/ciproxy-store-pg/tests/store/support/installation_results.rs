//! Public-store setup for pending installation results and session delivery.

use chrono::Duration;
use ciproxy_store_interface::dashboard::records::{Membership, Session, User};
use ciproxy_store_interface::installation_links::InstallationClaim;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use uuid::Uuid;

use super::Database;

pub(in crate::store) struct SignIn {
    pub(in crate::store) user: User,
    pub(in crate::store) memberships: Vec<Membership>,
    pub(in crate::store) session: Session,
}

pub(in crate::store) fn sign_in(database: &Database) -> SignIn {
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let now = "2026-11-07T12:00:00Z".parse().unwrap();
    let user = User {
        id: Uuid::new_v4(),
        github_id: 7,
        login: "person-7".to_owned(),
        avatar_url: "https://github.example/avatar".to_owned(),
        credentials: vec![11],
        checked_at: now,
    };
    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        digest: vec![12],
        csrf_digest: vec![13],
        expires_at: now + Duration::days(30),
    };
    SignIn {
        user,
        memberships: vec![Membership {
            github_id: organization.github_id,
            owner: false,
        }],
        session,
    }
}

pub(in crate::store) fn pending_result(
    database: &Database,
    installation: i64,
) -> InstallationClaim {
    let claim = InstallationClaim {
        id: Uuid::new_v4(),
        installation_id: installation,
        github_id: installation,
        login: "pending-result".to_owned(),
        requester_id: 7,
        event_seen: true,
        user_verified: true,
        direct: false,
        workspace_id: None,
        organization_id: None,
        verified_at: Some("2026-11-07T12:00:00Z".parse().unwrap()),
    };
    database
        .store
        .save_installation_claim(&claim, None, InstallationResultDelivery::Pending)
        .unwrap()
}
