//! A fresh start derives dispatch facts after the final key preparation wait.

use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifetime;

use super::dispatch_support::Provider;
use super::fixtures::{id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn upgrade_during_key_preparation_uses_paid_dispatch_budget_and_lifetime() {
    let provider = Provider::new(true);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    let facts = provider.facts.clone();
    harness.state.lock().unwrap().key_open_effect = Some((
        3,
        Box::new(move || {
            let mut facts = facts.lock().unwrap();
            facts.trial = false;
            facts.start_fee_bound_usd = Some(Decimal::new(7, 2));
        }),
    ));
    harness.set_limit(300_000);
    assert_eq!(
        harness.service.start(id(1), request()).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2026-12-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert!(!provider.facts.lock().unwrap().trial);
    let pending = harness.start(id(11));
    assert_eq!(pending.phase, StartPhase::Unsent);
    assert_eq!(pending.dispatch, None);
    assert_eq!(pending.start_fee_bound_usd, Some(Decimal::new(3, 2)));
    assert_eq!(provider.creates.lock().unwrap().len(), 0);
    assert_eq!(*provider.reads.lock().unwrap(), (2, 0));
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .budgets
            .last()
            .unwrap()
            .current_month_usd,
        Decimal::new(462, 3)
    );
    assert_eq!(
        harness
            .service
            .view(id(1))
            .await
            .unwrap()
            .spend
            .counted_usd()
            .unwrap(),
        Decimal::ZERO
    );
    harness.set_limit(462_000);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Pending(_)
    ));
    let saved = harness.start(id(11));
    assert_eq!(saved.delivery_id, pending.delivery_id);
    assert_eq!(saved.runner_id, pending.runner_id);
    assert_eq!(saved.start_fee_bound_usd, pending.start_fee_bound_usd);
    assert_eq!(saved.dispatch.as_ref().unwrap().lifetime_seconds, 21600);
    assert_eq!(
        provider.creates.lock().unwrap()[0].lifetime,
        SandboxLifetime::OneShot {
            max_lifetime: std::time::Duration::from_secs(21600)
        }
    );
    assert_eq!(
        harness
            .state
            .lock()
            .unwrap()
            .budgets
            .last()
            .unwrap()
            .current_month_usd,
        Decimal::new(462, 3)
    );
}
