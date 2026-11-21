//! Lifecycle snapshots commit with their state and survive mutable-row retention.

use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::github_lifecycle::{
    GithubLifecycleTarget, GithubLifecycleUpdate, LifecycleEvent,
};
use ciproxy_store_interface::organizations::AppState;
use ciproxy_store_interface::workspaces::Workspace;
use diesel::prelude::*;
use uuid::Uuid;

use super::super::support::{Database, at, hash};
use super::fixtures::registration;

#[test]
fn lifecycle_state_and_outbox_failures_roll_back_the_complete_transition() {
    let mut database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let token_hash = hash();
    database
        .store
        .create_install_token(workspace.id, Uuid::new_v4(), &token_hash, at())
        .unwrap();
    diesel::sql_query(
        "ALTER TABLE github_lifecycle_outbox ADD CONSTRAINT fail_delivery_insert CHECK (FALSE)",
    )
    .execute(&mut database.connection)
    .unwrap();
    let update = GithubLifecycleUpdate {
        github_id: organization.github_id,
        installation_id: organization.installation_id,
        login: "verified-rename".to_owned(),
        repositories: None,
        target: GithubLifecycleTarget::Suspended,
        notify: false,
    };
    assert!(matches!(
        database
            .store
            .update_github_installation(workspace.id, organization.id, &update),
        Err(Error::InvalidValue)
    ));
    assert_eq!(
        database
            .store
            .organization(workspace.id, organization.id)
            .unwrap(),
        Some(organization.clone())
    );
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(organization.github_id)
            .unwrap(),
        vec![]
    );
    assert!(matches!(
        database.store.remove_github_installation(
            workspace.id,
            organization.id,
            organization.installation_id,
            at()
        ),
        Err(Error::InvalidValue)
    ));
    assert_eq!(
        database
            .store
            .organization(workspace.id, organization.id)
            .unwrap(),
        Some(organization.clone())
    );
    assert!(
        matches!(database.store.authenticate_token(&token_hash, at()).unwrap(), Some(token) if token.revoked_at.is_none())
    );
    assert_eq!(
        database.store.workspace(workspace.id).unwrap(),
        Some(workspace)
    );
    assert_eq!(
        database
            .store
            .pending_github_lifecycle_organizations()
            .unwrap(),
        Vec::<i64>::new()
    );
    let new_workspace = Workspace {
        id: Uuid::new_v4(),
        name: "uncommitted".to_owned(),
        delete_after: None,
    };
    let mut replacement = organization.clone();
    replacement.id = Uuid::new_v4();
    replacement.workspace_id = new_workspace.id;
    replacement.github_id ^= 1;
    replacement.installation_id ^= 1;
    assert!(matches!(
        database.store.register_github_installation(&registration(
            Some(&new_workspace),
            &replacement,
            &[],
            41
        )),
        Err(Error::InvalidValue)
    ));
    assert_eq!(database.store.workspace(new_workspace.id).unwrap(), None);
    assert_eq!(
        database
            .store
            .pending_initial_admin(new_workspace.id)
            .unwrap(),
        None
    );
    assert_eq!(
        database
            .store
            .pending_github_lifecycle_organizations()
            .unwrap(),
        Vec::<i64>::new()
    );
    diesel::sql_query("ALTER TABLE github_lifecycle_outbox DROP CONSTRAINT fail_delivery_insert")
        .execute(&mut database.connection)
        .unwrap();
    let mut stale = update.clone();
    stale.installation_id ^= 1;
    assert!(matches!(
        database.store.update_github_installation(
            organization.workspace_id,
            organization.id,
            &stale
        ),
        Err(Error::NotFound)
    ));
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(organization.github_id)
            .unwrap(),
        vec![]
    );
    let saved = database
        .store
        .update_github_installation(organization.workspace_id, organization.id, &update)
        .unwrap();
    let mut expected = organization;
    expected.login = "verified-rename".to_owned();
    expected.app_state = AppState::Suspended;
    assert_eq!(saved, expected.clone());
    assert_eq!(
        database
            .store
            .pending_github_lifecycle(expected.github_id)
            .unwrap()
            .into_iter()
            .map(|delivery| delivery.event)
            .collect::<Vec<_>>(),
        vec![LifecycleEvent::Suspended(expected)]
    );
}
