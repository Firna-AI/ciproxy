//! Workspace return URLs through the authorized billing service boundary.

use std::sync::{Arc, Mutex};

use chrono::{DateTime, Utc};
use ciproxy_store_interface::billing::commands::{BillingCommand, CommandKind};
use ciproxy_store_interface::billing::state::{AddOns, BillingState, Selection, Tier};
use ciproxy_store_interface::billing::store::{
    BillingRecord, BillingSession, BillingSessionMock, BillingStoreMock,
};
use unimock::{MockFn, Unimock, matching};
use uuid::Uuid;

use crate::access::{BillingAccess, Role};
use crate::alerts::HostedBillingAlerts;
use crate::clock::ClockMock;
use crate::service::api::BillingApi;
use crate::service::runtime::{BillingService, Redirects};
use crate::stripe::api::StripeApiMock;
use crate::stripe::wire::types::{CheckoutSession, CheckoutStatus, Metadata, PaymentStatus};
use crate::webhook::SigningSecret;

#[test]
fn checkout_and_portal_returns_bind_the_paying_workspace_and_preserve_legacy_urls() {
    let templates = [
        "https://app.example/checkout-return?workspace={WORKSPACE_ID}&session_id={CHECKOUT_SESSION_ID}",
        "https://app.example/plans?workspace={WORKSPACE_ID}",
        "https://app.example/billing?workspace={WORKSPACE_ID}",
    ];
    let legacy = [
        "https://app.example/checkout-return?session_id={CHECKOUT_SESSION_ID}",
        "https://app.example/plans",
        "https://app.example/billing",
    ];
    for (workspace, configured, expected) in [
        (
            "11111111-1111-4111-8111-111111111111",
            templates,
            [
                "https://app.example/checkout-return?workspace=11111111-1111-4111-8111-111111111111&session_id={CHECKOUT_SESSION_ID}",
                "https://app.example/plans?workspace=11111111-1111-4111-8111-111111111111",
                "https://app.example/billing?workspace=11111111-1111-4111-8111-111111111111",
            ],
        ),
        (
            "33333333-3333-4333-8333-333333333333",
            templates,
            [
                "https://app.example/checkout-return?workspace=33333333-3333-4333-8333-333333333333&session_id={CHECKOUT_SESSION_ID}",
                "https://app.example/plans?workspace=33333333-3333-4333-8333-333333333333",
                "https://app.example/billing?workspace=33333333-3333-4333-8333-333333333333",
            ],
        ),
        ("11111111-1111-4111-8111-111111111111", legacy, legacy),
    ] {
        let workspace = workspace.parse().unwrap();
        let service = service(workspace, configured, expected);
        let access = BillingAccess {
            workspace_id: workspace,
            role: Role::Admin,
        };
        service.open_checkout(access, selection()).unwrap();
        service.portal(access).unwrap();
    }
}

fn service(
    workspace: Uuid,
    configured: [&str; 3],
    expected: [&'static str; 3],
) -> Box<dyn BillingApi> {
    let now: DateTime<Utc> = "2026-11-08T12:00:00Z".parse().unwrap();
    let command_id = "22222222-2222-4222-8222-222222222222".parse().unwrap();
    let initial = BillingRecord {
        state: BillingState::free(workspace, now),
        command: None,
        checkout: None,
        removal_cancel_requested: false,
    };
    let mut persisted = initial.clone();
    persisted.command = Some(BillingCommand {
        id: command_id,
        started_at: now,
        payment_expires_at: None,
        kind: CommandKind::Checkout {
            selection: selection(),
            success_url: expected[0].into(),
            cancel_url: expected[1].into(),
        },
    });
    let mut paid = initial.clone();
    paid.state.tier = Tier::Team;
    paid.state.customer_id = Some("cus_redirects".into());
    let session: Box<dyn BillingSession> = Box::new(Unimock::new((
        BillingSessionMock::record.stub(|calls| {
            calls
                .call(matching!())
                .answers_arc(Arc::new(move |_| Ok(initial.clone())))
                .once()
                .then()
                .answers_arc(Arc::new(move |_| Ok(persisted.clone())));
        }),
        BillingSessionMock::save_command
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, command| {
                assert_checkout_urls(command, expected);
                Ok(())
            })),
        BillingSessionMock::save_checkout
            .some_call(matching!(_))
            .returns(Ok(())),
        BillingSessionMock::commit
            .some_call(matching!(_, _, _, _, _))
            .returns(Ok(())),
    )));
    let stripe = Unimock::new((
        StripeApiMock::checkout
            .some_call(matching!(_, None, _))
            .answers_arc(Arc::new(move |_, actual_workspace, _, command| {
                assert_eq!(actual_workspace, workspace);
                assert_checkout_urls(command, expected);
                Ok(CheckoutSession {
                    id: "cs_redirects".into(),
                    status: CheckoutStatus::Open,
                    payment_status: PaymentStatus::Unpaid,
                    url: Some("https://checkout.stripe.test/cs_redirects".into()),
                    customer: None,
                    subscription: None,
                    metadata: Metadata {
                        workspace_id: Some(workspace),
                        command_id: Some(command_id),
                        ..Metadata::default()
                    },
                })
            })),
        StripeApiMock::portal
            .some_call(matching!("cus_redirects", _))
            .answers_arc(Arc::new(move |_, _, return_url| {
                assert_eq!(return_url, expected[2], "portal return URL");
                Ok("https://billing.stripe.test/redirects".into())
            })),
    ));
    let session = Mutex::new(Some(session));
    Box::new(BillingService::new(
        Arc::new(Unimock::new((
            BillingStoreMock::try_lock_billing
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, actual_workspace| {
                    assert_eq!(actual_workspace, workspace);
                    Ok(session.lock().unwrap().take())
                })),
            BillingStoreMock::billing
                .some_call(matching!(_))
                .returns(Ok(paid)),
        ))),
        Arc::new(stripe),
        Arc::new(Unimock::new((
            ClockMock::now.each_call(matching!()).returns(now),
            ClockMock::command_id
                .some_call(matching!())
                .returns(command_id),
        ))),
        Arc::new(HostedBillingAlerts),
        SigningSecret::new("whsec_offline_only".into()),
        Redirects {
            success_url: configured[0].into(),
            cancel_url: configured[1].into(),
            portal_return_url: configured[2].into(),
        },
    ))
}

fn selection() -> Selection {
    Selection {
        tier: Tier::Team,
        add_ons: AddOns::default(),
    }
}

fn assert_checkout_urls(command: &BillingCommand, expected: [&str; 3]) {
    let CommandKind::Checkout {
        success_url,
        cancel_url,
        ..
    } = &command.kind
    else {
        panic!("expected a Checkout command");
    };
    assert_eq!(success_url, expected[0], "Checkout success URL");
    assert_eq!(cancel_url, expected[1], "Checkout cancel URL");
}
