//! Current saved limits govern reservation after launch encryption finishes.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::provider::BoatAccountApiMock;
use ciproxy_store_interface::overflow::admission::ReservationBudget;
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifecycleMock;
use unimock::{MockFn, Unimock, matching};

use super::fixtures::{capabilities, facts, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn limit_lowered_during_encryption_refuses_0502_dollars_under_010_dollars() {
    let creates = Arc::new(AtomicUsize::new(0));
    let creating = creates.clone();
    let mut backend = capabilities(Unimock::new(
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| {
                let mut facts = facts();
                facts.start_fee_bound_usd = Some(Decimal::new(7, 2));
                Ok(facts)
            }),
    ));
    backend.lifecycle = Arc::new(
        Unimock::new(
            SandboxLifecycleMock::create_sandbox
                .some_call(matching!(_))
                .answers_arc(Arc::new(move |_, _| {
                    creating.fetch_add(1, Ordering::SeqCst);
                    Err(sandbox_interface::Error::BackendUnavailable {
                        backend_id: "boat".to_owned(),
                    })
                })),
        )
        .no_verify_in_drop(),
    );
    let harness = Harness::new(backend, (), Runtime::Start);
    harness.state.lock().unwrap().limit_on_launch_seal =
        Some((100_000, harness.database.store.clone()));
    let outcome = harness.service.start(id(1), request()).await.unwrap();
    let saved = harness.snapshot();
    assert_eq!(saved.spend_limit_usd_micros, Some(100_000));
    assert_eq!(saved.starts, []);
    assert_eq!(creates.load(Ordering::SeqCst), 0);
    assert_eq!(
        harness.state.lock().unwrap().budgets,
        [ReservationBudget {
            current_month_usd: Decimal::new(502, 3),
            next_month_usd: None,
        }]
    );
    assert_eq!(
        outcome,
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
}
