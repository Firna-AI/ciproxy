//! Lifecycle snapshots commit with their state and survive mutable-row retention.

use chrono::Duration;
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::github_lifecycle::{LifecycleEvent, RemovalCause};
use ciproxy_store_interface::organizations::AppState;

use super::super::support::{Database, at};
use super::fixtures::registration;

#[test]
fn pending_lifecycle_snapshots_survive_row_reuse_and_workspace_purge_until_confirmation() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let github_id = organization.github_id;
    let removed = database
        .store
        .remove_github_installation(
            workspace.id,
            organization.id,
            organization.installation_id,
            at(),
        )
        .unwrap();
    let mut expected_removed = organization.clone();
    expected_removed.app_state = AppState::Removed;
    expected_removed.removed_at = Some(at());
    expected_removed.delete_after = Some(at() + Duration::days(30));
    assert_eq!(removed, expected_removed.clone());
    let mut next = organization;
    next.installation_id ^= 1;
    let active = database
        .store
        .register_github_installation(&registration(None, &next, &[], 41))
        .unwrap();
    assert_eq!(active, next.clone());
    let removed_next = database
        .store
        .remove_github_installation(
            workspace.id,
            next.id,
            next.installation_id,
            at() + Duration::days(1),
        )
        .unwrap();
    let expected = vec![
        LifecycleEvent::Removed {
            organization: expected_removed,
            last_organization: true,
            cause: RemovalCause::Owner,
        },
        LifecycleEvent::Active(next),
        LifecycleEvent::Removed {
            organization: removed_next,
            last_organization: true,
            cause: RemovalCause::Owner,
        },
    ];
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(github_id)
            .unwrap()
            .into_iter()
            .map(|delivery| delivery.event)
            .collect::<Vec<_>>(),
        expected
    );
    assert!(
        database
            .store
            .purge_expired_workspace(workspace.id, at() + Duration::days(31))
            .unwrap()
    );
    assert_eq!(
        database
            .store
            .organizations_by_github_id(github_id)
            .unwrap(),
        vec![]
    );
    assert_eq!(
        database
            .store
            .pending_github_lifecycle_organizations()
            .unwrap(),
        vec![github_id]
    );
    let pending = database.store.pending_github_lifecycle(github_id).unwrap();
    assert_eq!(
        pending
            .iter()
            .map(|delivery| delivery.event.clone())
            .collect::<Vec<_>>(),
        expected
    );
    assert!(matches!(
        database
            .store
            .confirm_github_lifecycle(github_id ^ 1, pending[0].sequence),
        Err(Error::NotFound)
    ));
    for (index, delivery) in pending.iter().enumerate() {
        database
            .store
            .confirm_github_lifecycle(github_id, delivery.sequence)
            .unwrap();
        assert_eq!(
            database
                .store
                .pending_github_lifecycle(github_id)
                .unwrap()
                .into_iter()
                .map(|delivery| delivery.event)
                .collect::<Vec<_>>(),
            expected[index + 1..]
        );
    }
    assert_eq!(
        database
            .store
            .pending_github_lifecycle_organizations()
            .unwrap(),
        Vec::<i64>::new()
    );
}

#[test]
fn removal_outbox_keeps_owner_and_workspace_takeout_causes() {
    let database = Database::new();
    let workspace = database.workspace();
    let taken = database.organization(&workspace);
    let owner = database.organization(&workspace);
    database
        .store
        .begin_organization_takeout(workspace.id, taken.id)
        .unwrap();
    let mut expected_taken = taken.clone();
    expected_taken.app_state = AppState::Removed;
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(taken.github_id)
            .unwrap()[0]
            .event,
        LifecycleEvent::Removed {
            organization: expected_taken,
            last_organization: false,
            cause: RemovalCause::WorkspaceTakeout
        }
    );
    let removed = database
        .store
        .remove_github_installation(workspace.id, owner.id, owner.installation_id, at())
        .unwrap();
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(owner.github_id)
            .unwrap()[0]
            .event,
        LifecycleEvent::Removed {
            organization: removed,
            last_organization: true,
            cause: RemovalCause::Owner
        }
    );
}
