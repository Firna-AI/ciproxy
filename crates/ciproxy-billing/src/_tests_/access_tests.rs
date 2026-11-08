//! Authorization and review contracts through the public service boundary.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::state::{AddOns, Selection, Tier};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::access::{BillingAccess, Role};
use crate::alerts::HostedBillingAlerts;
use crate::clock::ClockMock;
use crate::error::Error;
use crate::service::api::BillingApi;
use crate::service::runtime::{BillingService, Redirects};
use crate::webhook::SigningSecret;

fn service(clock: Unimock) -> Box<dyn BillingApi> {
    Box::new(BillingService::new(
        Arc::new(Unimock::new(())),
        Arc::new(Unimock::new(())),
        Arc::new(clock),
        Arc::new(HostedBillingAlerts),
        SigningSecret::new("secret".into()),
        Redirects {
            success_url: "https://app.example/success".into(),
            cancel_url: "https://app.example/cancel".into(),
            portal_return_url: "https://app.example/billing".into(),
        },
    ))
}

#[test]
fn viewers_cannot_choose_pay_change_cancel_or_open_invoices() {
    let service = service(Unimock::new(()));
    let access = BillingAccess {
        workspace_id: Uuid::nil(),
        role: Role::Viewer,
    };
    let selection = Selection {
        tier: Tier::Team,
        add_ons: AddOns::default(),
    };
    assert!(matches!(
        service.review(access, selection),
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        service.open_checkout(access, selection),
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        service.reconcile_checkout(access, "cs_other"),
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        service.change(access, Uuid::nil()),
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        service.change_add_ons(access, AddOns::default()),
        Err(Error::Forbidden)
    ));
    assert!(matches!(
        service.move_to_free(access),
        Err(Error::Forbidden)
    ));
    assert!(matches!(service.portal(access), Err(Error::Forbidden)));
}

#[test]
fn payers_review_monthly_usd_totals_and_billing_day_without_stripe_customer() {
    let at: DateTime<Utc> = "2026-11-04T12:00:00Z".parse().unwrap();
    let service = service(Unimock::new(
        ClockMock::now.each_call(matching!()).returns(at),
    ));
    for (role, tier, blocks, machines, cents) in [
        (Role::Admin, Tier::Team, 0, 0, 4900),
        (Role::Billing, Tier::Business, 0, 0, 29900),
        (Role::Billing, Tier::Team, 2, 1, 8900),
    ] {
        let review = service
            .review(
                BillingAccess {
                    workspace_id: Uuid::nil(),
                    role,
                },
                Selection {
                    tier,
                    add_ons: AddOns {
                        start_blocks: blocks,
                        machines,
                    },
                },
            )
            .unwrap();
        assert_eq!(
            (review.monthly_cents, review.today_cents, review.billing_day),
            (cents, cents, 4)
        );
    }
}
