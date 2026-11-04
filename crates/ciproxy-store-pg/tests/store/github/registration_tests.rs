//! Atomic defaults and repository snapshots preserve tenant boundaries.

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::state::{AddOns, SubscriptionState, Tier};
use ciproxy_store_interface::error::Error;
use ciproxy_store_interface::github::Repository;
use ciproxy_store_interface::metering::history::{Period, PlanCost};
use ciproxy_store_interface::organizations::Organization;
use ciproxy_store_interface::workspaces::Workspace;
use diesel::RunQueryDsl;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::fixtures::registration;
use ciproxy_store_interface::github_lifecycle::{GithubLifecycleTarget, GithubPreparedResources};

use super::super::support::{Database, at};

#[test]
fn failed_registration_rolls_back_new_workspace_and_initial_admin() {
    let database = Database::new();
    let workspace = database.workspace();
    let organization = database.organization(&workspace);
    let new_workspace = Workspace {
        id: Uuid::new_v4(),
        name: "uncommitted".to_owned(),
        delete_after: None,
    };
    let wrong_workspace = Organization {
        workspace_id: new_workspace.id,
        ..organization.clone()
    };
    assert!(matches!(
        database.store.register_github_installation(&registration(
            Some(&new_workspace),
            &wrong_workspace,
            &[],
            41
        )),
        Err(Error::NotFound)
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
            .organization(workspace.id, organization.id)
            .unwrap(),
        Some(organization)
    );
}

#[test]
fn atomic_install_preserves_first_admin_and_replay_keeps_workspace_defaults() {
    let mut database = Database::new();
    diesel::sql_query("ALTER TABLE billing_accounts ALTER COLUMN created_at SET DEFAULT '2026-01-31T12:00:00Z'::timestamptz")
        .execute(&mut database.connection).unwrap();
    let creation: DateTime<Utc> = "2026-01-31T12:00:00Z".parse().unwrap();
    let initial_period = Period {
        starts_at: creation,
        ends_at: "2026-02-28T12:00:00Z".parse().unwrap(),
    };
    let source_workspace = database.workspace();
    let source_organization = database.organization(&source_workspace);
    let (mut label, _) = database.label_and_scale_set(&source_organization);
    label.id = Uuid::new_v4();
    label.name = "ciproxy".to_owned();
    let workspace = Workspace {
        id: Uuid::new_v4(),
        name: "customer".to_owned(),
        delete_after: None,
    };
    let mut organization = source_organization;
    organization.id = Uuid::new_v4();
    organization.workspace_id = workspace.id;
    organization.github_id ^= 1;
    organization.installation_id ^= 1;
    let store = &database.store;
    store
        .register_github_installation(&registration(
            Some(&workspace),
            &organization,
            &[label.clone()],
            41,
        ))
        .unwrap();
    assert_eq!(
        store.workspace(workspace.id).unwrap(),
        Some(workspace.clone())
    );
    let billing = store.billing(workspace.id);
    assert!(
        billing.is_ok(),
        "GitHub workspace must have Free billing: {billing:?}"
    );
    let billing = billing.unwrap();
    assert_eq!(billing.state.free_started_at, creation);
    assert_eq!(
        (
            billing.state.workspace_id,
            billing.state.tier,
            billing.state.add_ons,
            billing.state.status,
            billing.state.customer_id,
            billing.state.subscription_id
        ),
        (
            workspace.id,
            Tier::Free,
            AddOns {
                start_blocks: 0,
                machines: 0
            },
            SubscriptionState::None,
            None,
            None
        )
    );
    assert_eq!(
        (
            billing.command,
            billing.checkout,
            billing.removal_cancel_requested
        ),
        (None, None, false)
    );
    for workspace_id in [workspace.id, source_workspace.id] {
        let costs = store
            .savings_snapshot(
                workspace_id,
                "2026-01-01".parse().unwrap(),
                "2026-03-01".parse().unwrap(),
            )
            .unwrap()
            .costs;
        assert_eq!(costs.workspace_started_at, Some(creation));
        assert_eq!(
            costs.plans,
            vec![PlanCost {
                id: workspace_id,
                starts_at: creation,
                ends_at: None,
                monthly_usd: Decimal::ZERO
            }]
        );
        assert_eq!(
            store.metering_periods(workspace_id).unwrap(),
            vec![initial_period]
        );
    }
    assert_eq!(store.pending_initial_admin(workspace.id).unwrap(), Some(41));
    let changed_price = PlanCost {
        id: workspace.id,
        starts_at: creation,
        ends_at: None,
        monthly_usd: Decimal::new(7, 0),
    };
    let changed_period = Period {
        starts_at: creation,
        ends_at: "2026-02-14T12:00:00Z".parse().unwrap(),
    };
    store.save_plan_cost(workspace.id, &changed_price).unwrap();
    store
        .save_metering_period(workspace.id, &changed_period)
        .unwrap();
    label.size.vcpus = 6;
    store.update_label(workspace.id, &label).unwrap();
    store
        .register_github_installation(&registration(None, &organization, &[], 99))
        .unwrap();
    assert_eq!(store.labels(workspace.id).unwrap(), vec![label]);
    let replay_costs = store
        .savings_snapshot(
            workspace.id,
            "2026-01-01".parse().unwrap(),
            "2026-03-01".parse().unwrap(),
        )
        .unwrap()
        .costs;
    assert_eq!(replay_costs.workspace_started_at, Some(creation));
    assert_eq!(replay_costs.plans, vec![changed_price]);
    assert_eq!(
        store.metering_periods(workspace.id).unwrap(),
        vec![changed_period]
    );
    assert_eq!(store.pending_initial_admin(workspace.id).unwrap(), Some(41));
    store.confirm_initial_admin(workspace.id).unwrap();
    assert_eq!(store.pending_initial_admin(workspace.id).unwrap(), None);
    let repositories = vec![Repository {
        id: 23,
        full_name: "customer/private".to_owned(),
        private: true,
        default_branch: "trunk".to_owned(),
    }];
    store
        .replace_repositories(workspace.id, organization.id, &repositories)
        .unwrap();
    assert_eq!(
        store.repositories(workspace.id, organization.id).unwrap(),
        repositories
    );
    assert!(matches!(
        store.replace_repositories(source_workspace.id, organization.id, &[]),
        Err(Error::NotFound)
    ));
    assert_eq!(
        store.repositories(workspace.id, organization.id).unwrap(),
        repositories
    );
    store
        .record_app_removal(workspace.id, organization.id, at())
        .unwrap();
    let mut reinstall = registration(None, &organization, &[], 99);
    reinstall.target = GithubLifecycleTarget::Active(GithubPreparedResources {
        repositories: repositories.clone(),
        scale_sets: vec![],
    });
    store.register_github_installation(&reinstall).unwrap();
    assert_eq!(store.workspace(workspace.id).unwrap(), Some(workspace));
    assert_eq!(
        store
            .repositories(organization.workspace_id, organization.id)
            .unwrap(),
        repositories
    );
}
