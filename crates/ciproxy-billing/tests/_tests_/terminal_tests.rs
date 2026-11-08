//! Ended subscriptions release paid commands without releasing a new Checkout.

use ciproxy_billing::error::Error;
use ciproxy_billing::service::view::{CheckoutOutcome, EventOutcome};
use ciproxy_billing::stripe::wire::types::{PendingUpdate, Status};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{SubscriptionState, Tier};

use super::database::Database;
use super::fake::Fake;
use super::fixture::{
    access, at, command, read_service, seed_paid, service, signature, state, subscription, wanted,
    workspace,
};
use super::quote_fixture::quote;

fn prepare(cancel: bool) -> (Database, Fake) {
    let database = Database::new(workspace());
    seed_paid(&database);
    let mut current = subscription();
    current.status = Status::Canceled;
    current.ended_at = Some(1793880000);
    current.pending_update = Some(PendingUpdate {
        expires_at: 1793966400,
    });
    let fake = Fake::new(current.clone());
    {
        let mut lock = database
            .store
            .try_lock_billing(workspace())
            .unwrap()
            .unwrap();
        lock.save_command(&BillingCommand {
            id: command(),
            started_at: at(),
            payment_expires_at: Some("2026-11-06T12:00:00Z".parse().unwrap()),
            kind: if cancel {
                CommandKind::Cancel
            } else {
                CommandKind::Change {
                    selection: wanted(Tier::Business, 2, 1),
                    upgrade: true,
                    quote: None,
                }
            },
        })
        .unwrap();
    }
    (database, fake)
}

#[test]
fn ended_events_release_paid_commands_and_preserve_a_later_checkout() {
    for cancel in [false, true] {
        let (database, fake) = prepare(cancel);
        let now = "2026-11-05T12:00:00Z".parse().unwrap();
        let billing = service(&database, &fake, now);
        let raw = include_bytes!("fixtures/subscription_ended.json");
        assert_eq!(
            billing.webhook(&signature(raw, now), raw).unwrap(),
            EventOutcome::Applied
        );
        assert_eq!(database.store.billing(workspace()).unwrap().command, None);
        assert_eq!(
            (state(&database).tier, state(&database).status),
            (Tier::Free, SubscriptionState::Ended)
        );
        assert_eq!(state(&database).free_started_at, now);
        assert!(matches!(
            billing.open_checkout(access(), wanted(Tier::Team, 0, 0)),
            Ok(CheckoutOutcome::Open { .. })
        ));
        let checkout = database.store.billing(workspace()).unwrap();
        let late = String::from_utf8(raw.to_vec())
            .unwrap()
            .replace("evt_subscription_ended", "evt_old_end_late");
        assert_eq!(
            billing
                .webhook(&signature(late.as_bytes(), now), late.as_bytes())
                .unwrap(),
            EventOutcome::Applied
        );
        assert_eq!(database.store.billing(workspace()).unwrap(), checkout);
        assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 0);
    }
}

#[test]
fn direct_paid_reconciliation_commits_terminal_state_before_any_mutation() {
    for cancel in [true, false] {
        let (database, fake) = prepare(cancel);
        let now = "2026-11-05T12:00:00Z".parse().unwrap();
        let billing = read_service(&database, &fake, now);
        let result = if cancel {
            billing.move_to_free(access())
        } else {
            billing
                .change(access(), quote(&database, &fake, super::fixture::at()))
                .map(|_| ())
        };
        assert_eq!(
            result
                .err()
                .map(|error| matches!(error, Error::ReviewRequired)),
            if cancel { None } else { Some(true) }
        );
        assert_eq!(
            (state(&database).tier, state(&database).status),
            (Tier::Free, SubscriptionState::Ended)
        );
        assert_eq!(database.store.billing(workspace()).unwrap().command, None);
        assert_eq!(fake.posts("/v1/subscriptions/sub_recorded").len(), 0);
        let checkout = service(&database, &fake, now);
        assert!(matches!(
            checkout.open_checkout(access(), wanted(Tier::Team, 0, 0)),
            Ok(CheckoutOutcome::Open { .. })
        ));
    }
}
