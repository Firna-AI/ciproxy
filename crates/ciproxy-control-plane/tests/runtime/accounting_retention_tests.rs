//! Production accounting deletes only expired owned data after current refresh.

use std::sync::Arc;

use chrono::Duration;
use ciproxy_control_plane::accounting::Accounting;
use ciproxy_metering::service::StoredMetering;
use ciproxy_store_interface::error::Error;
use uuid::Uuid;

use super::accounting_retention_support::{
    accounting, cost, expected_starts, expected_totals, expired_demand, history, organization,
};
use super::support::{Fixture, at};

#[tokio::test]
async fn accounting_purges_due_organizations_and_workspaces_after_preserving_exact_totals() {
    let fixture = Fixture::new();
    let store = &fixture.store;
    let removed = history(
        &fixture,
        store
            .organization(fixture.workspace, fixture.organization)
            .unwrap()
            .unwrap(),
        true,
    );
    let sibling = history(
        &fixture,
        organization(&fixture, fixture.workspace, 90),
        false,
    );
    let independent = history(&fixture, organization(&fixture, Uuid::new_v4(), 100), true);
    let last = history(&fixture, organization(&fixture, Uuid::new_v4(), 110), true);
    let empty = organization(&fixture, Uuid::new_v4(), 120);
    expired_demand(&fixture);
    store
        .record_app_removal(fixture.workspace, removed.organization.id, at())
        .unwrap();
    store
        .record_app_removal(
            last.organization.workspace_id,
            last.organization.id,
            at() + Duration::days(1),
        )
        .unwrap();
    store
        .record_app_removal(empty.workspace_id, empty.id, at())
        .unwrap();
    let settings = store.settings(fixture.workspace).unwrap();
    let machines = store.machines(fixture.workspace).unwrap();
    let independent_workspace = store
        .workspace(independent.organization.workspace_id)
        .unwrap();
    let independent_settings = store
        .settings(independent.organization.workspace_id)
        .unwrap();
    let (worker, now) = accounting(&fixture, Arc::new(StoredMetering::new(store.clone())));
    *now.lock().unwrap() = at() + Duration::days(30) - Duration::microseconds(1);
    worker.pass().await;
    assert_eq!(
        store
            .github_jobs(fixture.workspace, removed.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(store.organizations(fixture.workspace).unwrap().len(), 2);
    assert_eq!(
        store
            .workspace(empty.workspace_id)
            .unwrap()
            .unwrap()
            .delete_after,
        Some(at() + Duration::days(30))
    );
    let mut corrected = removed.job.clone();
    corrected.completed_at = Some(at() + Duration::minutes(6));
    store.update_job(fixture.workspace, &corrected).unwrap();
    store
        .save_boat_cost(fixture.workspace, &cost(removed.runner.id, 11))
        .unwrap();
    assert_eq!(
        store
            .purge_removed_organizations(empty.workspace_id, at() + Duration::days(30))
            .unwrap(),
        1
    );
    assert_eq!(store.organizations(empty.workspace_id).unwrap(), vec![]);
    *now.lock().unwrap() = at() + Duration::days(30);
    worker.pass().await;
    assert_eq!(
        store
            .organization(fixture.workspace, removed.organization.id)
            .unwrap(),
        None
    );
    assert_eq!(
        store
            .jobs(fixture.workspace, removed.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(
        store
            .runners(fixture.workspace, removed.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(
        store
            .scale_sets(fixture.workspace, removed.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(store.workspace(empty.workspace_id).unwrap(), None);
    assert!(matches!(
        store.settings(empty.workspace_id),
        Err(Error::NotFound)
    ));
    assert_eq!(
        store.organizations(fixture.workspace).unwrap(),
        vec![sibling.organization.clone()]
    );
    assert_eq!(
        store
            .jobs(fixture.workspace, sibling.organization.id)
            .unwrap(),
        vec![sibling.job.clone()]
    );
    assert_eq!(
        store
            .runners(fixture.workspace, sibling.organization.id)
            .unwrap(),
        vec![sibling.runner.clone()]
    );
    assert_eq!(
        store
            .scale_sets(fixture.workspace, sibling.organization.id)
            .unwrap(),
        vec![sibling.scale_set.clone()]
    );
    assert_eq!(
        store.labels(fixture.workspace).unwrap(),
        vec![sibling.label.clone()]
    );
    assert_eq!(store.settings(fixture.workspace).unwrap(), settings);
    assert_eq!(store.machines(fixture.workspace).unwrap(), machines);
    let mut totals = expected_totals(&fixture, true);
    totals[0].sizes[0].boat.minutes = 6.into();
    totals[0].boat_usd = rust_decimal::Decimal::new(32, 2);
    let day = at().date_naive();
    assert_eq!(
        store
            .savings_snapshot(fixture.workspace, day, day.succ_opt().unwrap())
            .unwrap()
            .days,
        totals
    );
    assert_eq!(
        store.workspace_usage(fixture.workspace).unwrap().starts,
        expected_starts(true)
    );
    assert_eq!(
        store
            .workspace(last.organization.workspace_id)
            .unwrap()
            .unwrap()
            .delete_after,
        Some(at() + Duration::days(31))
    );
    assert_eq!(
        store
            .jobs(last.organization.workspace_id, last.organization.id)
            .unwrap(),
        vec![last.job.clone()]
    );
    *now.lock().unwrap() = at() + Duration::days(31);
    worker.pass().await;
    assert_eq!(
        store.workspace(last.organization.workspace_id).unwrap(),
        None
    );
    assert_eq!(
        store
            .organization(last.organization.workspace_id, last.organization.id)
            .unwrap(),
        None
    );
    assert_eq!(
        store.labels(last.organization.workspace_id).unwrap(),
        vec![]
    );
    assert!(matches!(
        store.settings(last.organization.workspace_id),
        Err(Error::NotFound)
    ));
    assert_eq!(
        store
            .jobs(last.organization.workspace_id, last.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(
        store
            .runners(last.organization.workspace_id, last.organization.id)
            .unwrap(),
        vec![]
    );
    assert_eq!(
        store
            .workspace_usage(last.organization.workspace_id)
            .unwrap()
            .starts,
        vec![]
    );
    assert_eq!(
        store
            .savings_snapshot(last.organization.workspace_id, day, day.succ_opt().unwrap())
            .unwrap()
            .days,
        vec![]
    );
    assert_eq!(
        store
            .workspace(independent.organization.workspace_id)
            .unwrap(),
        independent_workspace
    );
    assert_eq!(
        store
            .settings(independent.organization.workspace_id)
            .unwrap(),
        independent_settings
    );
    assert_eq!(
        store
            .jobs(
                independent.organization.workspace_id,
                independent.organization.id
            )
            .unwrap(),
        vec![independent.job]
    );
    assert_eq!(
        store
            .runners(
                independent.organization.workspace_id,
                independent.organization.id
            )
            .unwrap(),
        vec![independent.runner]
    );
    assert_eq!(
        store
            .organizations(independent.organization.workspace_id)
            .unwrap(),
        vec![independent.organization.clone()]
    );
    assert_eq!(
        store
            .savings_snapshot(
                independent.organization.workspace_id,
                day,
                day.succ_opt().unwrap()
            )
            .unwrap()
            .days,
        expected_totals(&fixture, false)
    );
    assert_eq!(
        store
            .workspace_usage(independent.organization.workspace_id)
            .unwrap()
            .starts,
        expected_starts(false)
    );
    assert_eq!(
        store
            .savings_snapshot(fixture.workspace, day, day.succ_opt().unwrap())
            .unwrap()
            .days,
        totals
    );
    assert_eq!(
        store
            .workspace_usage(fixture.workspace)
            .unwrap()
            .accounted_jobs
            .len(),
        1
    );
    assert_eq!(
        store.boat_accounting(fixture.workspace).unwrap().accounted,
        vec![]
    );
}
