//! Trusted service composition and literal recorded-shape test data.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_billing::access::{BillingAccess, Role};
use ciproxy_billing::alerts::HostedBillingAlerts;
use ciproxy_billing::clock::{Clock, ClockMock, SystemClock};
use ciproxy_billing::service::api::BillingApi;
use ciproxy_billing::service::runtime::{BillingService, Redirects};
use ciproxy_billing::stripe::http::client::HttpStripe;
use ciproxy_billing::stripe::wire::types::{
    CheckoutSession, CheckoutStatus, PaymentStatus, Subscription,
};
use ciproxy_billing::webhook::SigningSecret;
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{AddOns, BillingState, Selection, Tier};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use super::checked_stripe::CheckedStripe;
use super::database::Database;
use super::fake::Fake;

pub(super) fn workspace() -> Uuid {
    "11111111-1111-4111-8111-111111111111".parse().unwrap()
}
pub(super) fn command() -> Uuid {
    "22222222-2222-4222-8222-222222222222".parse().unwrap()
}
pub(super) fn at() -> DateTime<Utc> {
    "2026-11-04T12:00:00Z".parse().unwrap()
}
pub(super) fn access() -> BillingAccess {
    BillingAccess {
        workspace_id: workspace(),
        role: Role::Admin,
    }
}
pub(super) fn wanted(tier: Tier, blocks: u32, machines: u32) -> Selection {
    Selection {
        tier,
        add_ons: AddOns {
            start_blocks: blocks,
            machines,
        },
    }
}
pub(super) fn subscription() -> Subscription {
    serde_json::from_str(include_str!("fixtures/subscription.json")).unwrap()
}

pub(super) fn service(database: &Database, fake: &Fake, now: DateTime<Utc>) -> Box<dyn BillingApi> {
    compose(
        database,
        fake,
        Arc::new(Unimock::new((
            ClockMock::now.each_call(matching!()).returns(now),
            ClockMock::command_id
                .each_call(matching!())
                .answers(&|_| Uuid::new_v4()),
        ))),
    )
}

pub(super) fn read_service(
    database: &Database,
    fake: &Fake,
    now: DateTime<Utc>,
) -> Box<dyn BillingApi> {
    compose(
        database,
        fake,
        Arc::new(Unimock::new(
            ClockMock::now.each_call(matching!()).returns(now),
        )),
    )
}

pub(super) fn idle_service(database: &Database, fake: &Fake) -> Box<dyn BillingApi> {
    compose(database, fake, Arc::new(SystemClock))
}

fn compose(database: &Database, fake: &Fake, clock: Arc<dyn Clock>) -> Box<dyn BillingApi> {
    Box::new(BillingService::new(
        database.store.clone(),
        Arc::new(CheckedStripe {
            provider: Arc::new(
                HttpStripe::with_base_url("sk_test_offline_only".into(), &fake.url).unwrap(),
            ),
            probe: fake.probe(),
        }),
        clock,
        Arc::new(HostedBillingAlerts),
        SigningSecret::new("whsec_offline_only".into()),
        Redirects {
            success_url: "https://app.example/return?session_id={CHECKOUT_SESSION_ID}".into(),
            cancel_url: "https://app.example/plans".into(),
            portal_return_url: "https://app.example/billing".into(),
        },
    ))
}

pub(super) fn seed_paid(database: &Database) {
    let mut session = database
        .store
        .try_lock_billing(workspace())
        .unwrap()
        .unwrap();
    let mut state = session.record().unwrap().state;
    state.tier = Tier::Team;
    state.add_ons = AddOns {
        start_blocks: 2,
        machines: 1,
    };
    state.customer_id = Some("cus_recorded".into());
    state.subscription_id = Some("sub_recorded".into());
    state.status = ciproxy_store_interface::billing::state::SubscriptionState::Active;
    state.paid_period_start = Some(at());
    state.paid_period_end = Some("2026-12-04T12:00:00Z".parse().unwrap());
    session
        .commit(Some(&state), None, true, false, at())
        .unwrap();
}

pub(super) fn seed_checkout(database: &Database, fake: &Fake) {
    let mut lock = database
        .store
        .try_lock_billing(workspace())
        .unwrap()
        .unwrap();
    lock.save_command(&BillingCommand {
        id: command(),
        started_at: at(),
        payment_expires_at: None,
        kind: CommandKind::Checkout {
            selection: wanted(Tier::Team, 2, 1),
            success_url: "https://app.example/return".into(),
            cancel_url: "https://app.example/plans".into(),
        },
    })
    .unwrap();
    let mut session: CheckoutSession =
        serde_json::from_str(include_str!("fixtures/checkout.json")).unwrap();
    session.status = CheckoutStatus::Complete;
    session.payment_status = PaymentStatus::Paid;
    session.customer = Some("cus_recorded".into());
    session.subscription = Some("sub_recorded".into());
    fake.state.lock().unwrap().checkout = Some(session);
}

pub(super) fn signature(raw: &[u8], now: DateTime<Utc>) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(b"whsec_offline_only").unwrap();
    mac.update(format!("{}.", now.timestamp()).as_bytes());
    mac.update(raw);
    format!(
        "t={},v1={}",
        now.timestamp(),
        hex::encode(mac.finalize().into_bytes())
    )
}

pub(super) fn state(database: &Database) -> BillingState {
    database.store.billing(workspace()).unwrap().state
}
