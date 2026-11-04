//! Last-organization removal schedules billing even during unfinished payment.

use ciproxy_billing::service::view::EventOutcome;
use ciproxy_billing::stripe::wire::types::PendingUpdate;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::Tier;
use ciproxy_store_interface::organizations::{AppState, Organization, RepositoryAccess};
use uuid::Uuid;

use super::database::Database;
use super::fake::{Fake, field};
use super::fake_subscriptions::{SubscriptionOutcome, SubscriptionRequest};
use super::fixture::{
    at, command, seed_checkout, seed_paid, service, signature, state, subscription, wanted,
    workspace,
};

fn organization(github_id: i64) -> Organization {
    Organization {
        id: Uuid::new_v4(),
        workspace_id: workspace(),
        github_id,
        login: format!("org-{github_id}"),
        installation_id: github_id,
        app_state: AppState::Active,
        repositories: RepositoryAccess::AllPrivate,
        removed_at: None,
        delete_after: None,
    }
}

#[test]
fn only_last_removal_cancels_and_a_pending_upgrade_cannot_block_it() {
    let database = Database::new(workspace());
    seed_paid(&database);
    let first = organization(10);
    let last = organization(11);
    database.store.install_organization(&first).unwrap();
    database.store.install_organization(&last).unwrap();
    let fake = Fake::new(subscription());
    let service = service(&database, &fake, at());
    database
        .store
        .set_app_suspended(workspace(), last.id, true)
        .unwrap();
    service.reconcile_removal(workspace()).unwrap();
    assert_eq!(
        database.store.billing_cancellations().unwrap(),
        Vec::<Uuid>::new()
    );
    database
        .store
        .record_app_removal(workspace(), first.id, at())
        .unwrap();
    service.reconcile_removal(workspace()).unwrap();
    assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 0);
    {
        let mut session = database
            .store
            .try_lock_billing(workspace())
            .unwrap()
            .unwrap();
        session
            .save_command(&BillingCommand {
                id: command(),
                started_at: at(),
                payment_expires_at: None,
                kind: CommandKind::Change {
                    selection: wanted(Tier::Business, 2, 1),
                    upgrade: true,
                },
            })
            .unwrap();
    }
    let mut pending = subscription();
    pending.pending_update = Some(PendingUpdate {
        expires_at: 1793880000,
    });
    let mut canceled = pending.clone();
    canceled.cancel_at_period_end = true;
    {
        let mut remote = fake.state.lock().unwrap();
        remote.subscription = pending;
        remote.expect_subscription(
            SubscriptionRequest::CancelAtPeriodEnd,
            SubscriptionOutcome::respond(canceled),
        );
    }
    database
        .store
        .record_app_removal(workspace(), last.id, at())
        .unwrap();
    assert_eq!(
        database.store.billing_cancellations().unwrap(),
        vec![workspace()]
    );
    service.reconcile_removal(workspace()).unwrap();
    assert_eq!(
        database.store.billing_cancellations().unwrap(),
        Vec::<Uuid>::new()
    );
    assert_eq!(
        (state(&database).tier, state(&database).add_ons.start_blocks),
        (Tier::Team, 2)
    );
    assert_eq!(
        state(&database).cancel_at,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        field(
            &fake.posts("/v1/subscriptions/sub_recorded")[0],
            "cancel_at_period_end"
        ),
        "true"
    );
}

#[test]
fn last_removal_during_checkout_cancels_the_subscription_when_payment_confirms() {
    let database = Database::new(workspace());
    let org = organization(12);
    database.store.install_organization(&org).unwrap();
    let fake = Fake::new(subscription());
    seed_checkout(&database, &fake);
    let service = service(&database, &fake, at());
    database
        .store
        .record_app_removal(workspace(), org.id, at())
        .unwrap();
    service.reconcile_removal(workspace()).unwrap();
    assert_eq!(
        database.store.billing_cancellations().unwrap(),
        vec![workspace()]
    );
    let mut canceled = subscription();
    canceled.cancel_at_period_end = true;
    fake.expect_subscription(
        SubscriptionRequest::CancelAtPeriodEnd,
        SubscriptionOutcome::respond(canceled),
    );
    let raw = include_bytes!("fixtures/checkout_completed.json");
    assert_eq!(
        service.webhook(&signature(raw, at()), raw).unwrap(),
        EventOutcome::Applied
    );
    assert_eq!(state(&database).tier, Tier::Team);
    assert_eq!(
        state(&database).cancel_at,
        Some("2026-12-04T12:00:00Z".parse().unwrap())
    );
    assert_eq!(
        database.store.billing_cancellations().unwrap(),
        Vec::<Uuid>::new()
    );
}
