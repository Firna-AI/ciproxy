//! Revoked operation identities cannot consume a newer pending connection.

use uuid::Uuid;

use super::super::support::{Database, at};
use super::support::setup;

#[test]
fn old_connection_cannot_claim_a_new_grant_with_the_same_key_version_after_reinstall() {
    let database = Database::new();
    let (workspace_id, mut account, record) = setup(&database);
    let organization = database
        .store
        .organization(workspace_id, record.organization_id)
        .unwrap()
        .unwrap();
    account.account.id = Uuid::new_v4();
    let mut session = database.store.lock_overflow(workspace_id).unwrap();
    let old = session
        .begin_connection(account.account.id)
        .unwrap()
        .unwrap();
    database
        .store
        .record_app_removal(workspace_id, organization.id, at())
        .unwrap();
    database.store.install_organization(&organization).unwrap();
    let new = session
        .begin_connection(account.account.id)
        .unwrap()
        .unwrap();
    assert_ne!(old, new);
    assert!(
        !session
            .complete_connection(old, &account, 300_000_000)
            .unwrap()
    );
    assert_eq!(session.snapshot().unwrap().accounts, []);
    assert!(
        session
            .complete_connection(new, &account, 300_000_000)
            .unwrap()
    );
    assert_eq!(session.snapshot().unwrap().accounts, [account.clone()]);
    assert!(
        !session
            .complete_connection(new, &account, 300_000_000)
            .unwrap()
    );
}

#[test]
fn pending_connections_keep_suspended_and_shared_workspace_app_permission() {
    for partial_removal in [false, true] {
        let database = Database::new();
        let (workspace_id, mut account, record) = setup(&database);
        account.account.id = Uuid::new_v4();
        let workspace = database.store.workspace(workspace_id).unwrap().unwrap();
        let mut session = database.store.lock_overflow(workspace_id).unwrap();
        if !partial_removal {
            database
                .store
                .set_app_suspended(workspace_id, record.organization_id, true)
                .unwrap();
        } else {
            database.organization(&workspace);
        }
        let grant = session.begin_connection(account.account.id).unwrap();
        assert_ne!(grant, None, "the workspace still has App permission");
        let grant = grant.unwrap();
        if partial_removal {
            database
                .store
                .record_app_removal(workspace_id, record.organization_id, at())
                .unwrap();
        }
        assert!(
            session
                .complete_connection(grant, &account, 300_000_000)
                .unwrap()
        );
        assert_eq!(
            session
                .snapshot()
                .unwrap()
                .accounts
                .iter()
                .filter(|account| account.account.retired_at.is_none())
                .map(|account| account.account.id)
                .collect::<Vec<_>>(),
            [account.account.id]
        );
    }
}
