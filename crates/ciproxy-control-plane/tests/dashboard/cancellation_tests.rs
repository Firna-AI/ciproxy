//! Durable removal requests survive busy billing, provider failure, and restart.

use ciproxy_api::runtime::Runtime;
use ciproxy_store_interface::billing::commands::CommandKind;
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::workspaces::Workspace;
use uuid::Uuid;

use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::support::{Dashboard, composition};

#[tokio::test]
async fn durable_cancellations_recover_after_restart_and_continue_past_busy_workspaces() {
    let dashboard = Dashboard::new().await;
    dashboard.save(&dashboard.paid());
    let store = &dashboard.fixture.database.store;
    let workspace = dashboard.fixture.workspace;
    let now = Runtime::now(&*dashboard.fixture.runtime);
    let second = Uuid::new_v4();
    store
        .create_workspace(&Workspace {
            id: second,
            name: "Second removal".into(),
            delete_after: None,
        })
        .unwrap();
    let mut organization = dashboard.fixture.organization.clone();
    organization.id = Uuid::new_v4();
    organization.workspace_id = second;
    organization.github_id = 200;
    organization.installation_id = 201;
    store.install_organization(&organization).unwrap();
    store
        .record_app_removal(workspace, dashboard.fixture.organization.id, now)
        .unwrap();
    store
        .record_app_removal(second, organization.id, now)
        .unwrap();
    assert!(store.billing(workspace).unwrap().removal_cancel_requested);
    assert!(store.billing(second).unwrap().removal_cancel_requested);
    {
        let _busy = store.try_lock_billing(workspace).unwrap().unwrap();
        dashboard.pages.cancellations.reconcile().await.unwrap();
        assert!(store.billing(workspace).unwrap().removal_cancel_requested);
        assert!(!store.billing(second).unwrap().removal_cancel_requested);
    }
    dashboard.fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::Reject(503),
    );
    dashboard.pages.cancellations.reconcile().await.unwrap();
    let failed = store.billing(workspace).unwrap();
    assert!(failed.removal_cancel_requested);
    assert!(matches!(
        failed.command.as_ref().unwrap().kind,
        CommandKind::Cancel
    ));
    let mut canceled = dashboard.fake.state.lock().unwrap().subscription.clone();
    canceled.cancel_at_period_end = true;
    dashboard.fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::respond(canceled),
    );
    let restarted = composition(
        &dashboard.fixture,
        &dashboard.fake,
        dashboard.alerts.clone(),
    )
    .await;
    restarted.cancellations.reconcile().await.unwrap();
    let saved = store.billing(workspace).unwrap();
    assert!(!saved.removal_cancel_requested);
    assert_eq!(saved.command, None);
    assert_eq!(saved.state.tier, Tier::Team);
    assert_eq!(saved.state.cancel_at, saved.state.paid_period_end);
    assert_eq!(store.billing_cancellations().unwrap(), Vec::<Uuid>::new());
    let requests = dashboard.fake.subscription_mutations();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].key, requests[1].key);
    dashboard.fake.probe().check();
    tokio::task::spawn_blocking(move || drop(restarted))
        .await
        .unwrap();
    dashboard.close().await;
}
