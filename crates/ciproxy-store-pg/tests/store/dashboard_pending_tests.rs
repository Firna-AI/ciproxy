//! Exact pending snapshots survive concurrent proof and failed transactional writes.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::installation_results::InstallationResultDelivery;
use ciproxy_store_interface::store::StoreFactory;
use ciproxy_store_pg::factory::PgStoreFactory;
use diesel::connection::SimpleConnection;
use uuid::Uuid;

use super::support::Database;
use super::support::installation_results::{SignIn, pending_result, sign_in};

#[test]
fn session_consumes_only_snapshotted_pending_revisions_and_keeps_later_results() {
    let database = Database::new();
    let SignIn {
        user,
        memberships,
        session,
    } = sign_in(&database);
    let first = pending_result(&database, 1066);
    let snapshot = database.store.pending_installation_results(7).unwrap();
    let peer = PgStoreFactory.connect(&database.url).unwrap();
    let mut other_requester = first.clone();
    other_requester.id = Uuid::new_v4();
    other_requester.installation_id = 3066;
    other_requester.github_id = 306;
    other_requester.requester_id = 8;
    peer.save_installation_claim(&other_requester, None, InstallationResultDelivery::Pending)
        .unwrap();
    let other_pending = peer.pending_installation_results(8).unwrap();
    assert_eq!(
        other_pending
            .iter()
            .map(|result| result.claim_id)
            .collect::<Vec<_>>(),
        vec![other_requester.id]
    );
    let later = pending_result(&database, 2066);
    peer.save_installation_claim(&first, None, InstallationResultDelivery::Pending)
        .unwrap();
    let pending = database.store.pending_installation_results(7).unwrap();
    assert_ne!(
        pending
            .iter()
            .find(|result| result.claim_id == first.id)
            .unwrap()
            .revision,
        snapshot[0].revision
    );
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &snapshot, None)
            .unwrap()
    );
    assert_eq!(
        database
            .store
            .dashboard_continuation(session.id, user.checked_at)
            .unwrap(),
        vec![first.id]
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        pending
    );
    assert_eq!(pending.len(), 2);
    assert!(pending.iter().any(|result| result.claim_id == later.id));
    let memberships = database.store.dashboard_memberships(user.id).unwrap();
    assert!(
        database
            .store
            .establish_dashboard_session(&user, &memberships, &session, &pending, None)
            .unwrap()
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        vec![]
    );
    assert_eq!(peer.pending_installation_results(8).unwrap(), other_pending);
}

#[test]
fn failed_pending_result_write_preserves_the_unproved_claim() {
    let mut database = Database::new();
    let mut claim = pending_result(&database, 1066);
    claim.user_verified = false;
    database
        .store
        .save_installation_claim(&claim, None, InstallationResultDelivery::Untracked)
        .unwrap();
    let pending = database.store.pending_installation_results(7).unwrap();
    database
        .connection
        .batch_execute(
            "CREATE FUNCTION fail_pending_result() RETURNS trigger AS $$
         BEGIN RAISE EXCEPTION 'controlled pending result write failure'; END
         $$ LANGUAGE plpgsql;
         CREATE TRIGGER fail_pending_result BEFORE INSERT OR UPDATE ON installation_pending_results
         FOR EACH ROW EXECUTE FUNCTION fail_pending_result();",
        )
        .unwrap();
    claim.user_verified = true;
    let failure =
        database
            .store
            .save_installation_claim(&claim, None, InstallationResultDelivery::Pending);
    assert!(matches!(failure, Err(Error::Internal(_))));
    assert!(
        !database
            .store
            .installation_claim(claim.id)
            .unwrap()
            .unwrap()
            .user_verified
    );
    assert_eq!(
        database.store.pending_installation_results(7).unwrap(),
        pending
    );
}
