//! Saved refusals hold other unsent deliveries without losing their custody.

use chrono::Duration;
use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::error::Error;
use ciproxy_store_interface::overflow::record::{StartPhase, StartRecord};
use rust_decimal::Decimal;
use sandbox_interface::{
    BillingRefusalReason, PlanSizeRefusalReason, SandboxStartRefusal, SandboxStartRetry,
    TemporaryStartRefusalReason,
};

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{at, id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn unsent_deliveries_wait_for_saved_refusals_and_dispatch_once_after_recovery() {
    let retry_at = at() + Duration::seconds(90);
    let cases = [
        (
            SandboxStartRefusal::Temporary {
                reason: TemporaryStartRefusalReason::RateLimited,
                retry: SandboxStartRetry {
                    retry_after: Some(std::time::Duration::from_secs(60)),
                    next_start_at: Some(retry_at),
                },
            },
            WaitReason::RetryAt(retry_at),
        ),
        (
            SandboxStartRefusal::Billing {
                reason: BillingRefusalReason::BillingRequired,
            },
            WaitReason::BillingRequired,
        ),
        (
            SandboxStartRefusal::PlanSize {
                reason: PlanSizeRefusalReason::PlanRequired,
            },
            WaitReason::SizeUnavailable,
        ),
    ];
    for (refusal, wait) in cases {
        let provider = Provider::new(DisableAt::Never);
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        let mut other = request();
        other.delivery_id = id(21);
        other.runner_id = id(22);
        for action in [request(), other.clone()] {
            {
                let mut capture = harness.state.lock().unwrap();
                capture.fail_on_key_open = Some(capture.key_opens + 2);
            }
            assert!(matches!(
                harness.service.start(id(1), action).await,
                Err(Error::KmsHttp { status: 503 })
            ));
        }
        let pending = harness.start(id(21));
        assert_eq!(pending.phase, StartPhase::Unsent);
        let input = harness.database.session().encrypted_launch(id(21)).unwrap();
        let hash = harness.state.lock().unwrap().hashes[&id(21)].clone();
        assert_eq!(provider.counts.lock().unwrap().creates, 0);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(provider.counts.lock().unwrap().account_reads, 2);
        *provider.refusal.lock().unwrap() = Some(refusal.clone());
        assert_eq!(
            harness.service.resume(id(1), id(11)).await.unwrap(),
            StartOutcome::Waiting(WaitReason::Refused(refusal))
        );
        *provider.billing_ready.lock().unwrap() = false;
        assert_eq!(
            harness.service.resume(id(1), id(21)).await.unwrap(),
            StartOutcome::Waiting(wait.clone())
        );
        assert_eq!(harness.start(id(21)), pending);
        assert_eq!(
            harness.database.session().encrypted_launch(id(21)).unwrap(),
            input
        );
        assert_eq!(
            harness
                .database
                .store
                .overflow_token_workspace(&hash)
                .unwrap(),
            Some(id(1))
        );
        assert_eq!(provider.counts.lock().unwrap().creates, 1);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(
            harness
                .service
                .view(id(1))
                .await
                .unwrap()
                .spend
                .counted_usd()
                .unwrap(),
            Decimal::new(462, 3)
        );

        if wait == WaitReason::SizeUnavailable {
            *provider.size_available.lock().unwrap() = false;
            *harness.now.lock().unwrap() = at() + Duration::minutes(5);
            assert_catalog_wait(&harness, &pending).await;
        }
        assert_eq!(provider.counts.lock().unwrap().creates, 1);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        *provider.billing_ready.lock().unwrap() = true;
        *provider.size_available.lock().unwrap() = true;
        *harness.now.lock().unwrap() = if wait == WaitReason::SizeUnavailable {
            at() + Duration::minutes(10)
        } else {
            retry_at
        };
        assert!(matches!(
            harness.service.resume(id(1), id(21)).await.unwrap(),
            StartOutcome::Started(_)
        ));
        let dispatched = harness.start(id(21));
        assert_eq!(dispatched.delivery_id, pending.delivery_id);
        assert_eq!(dispatched.runner_id, pending.runner_id);
        assert_eq!(dispatched.size, pending.size);
        assert_eq!(dispatched.start_fee_bound_usd, Some(Decimal::new(3, 2)));
        assert_eq!(dispatched.starts_at, *harness.now.lock().unwrap());
        assert_eq!(provider.counts.lock().unwrap().creates, 2);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(provider.counts.lock().unwrap().launches, 1);
        assert_eq!(
            provider.counts.lock().unwrap().created_deliveries,
            [id(11), id(21)]
        );
        let expected_reads = match wait {
            WaitReason::RetryAt(_) => 2,
            WaitReason::BillingRequired | WaitReason::SizeUnavailable => 4,
            _ => unreachable!(),
        };
        assert_eq!(
            provider.counts.lock().unwrap().account_reads,
            expected_reads
        );
        assert!(matches!(
            harness.service.start(id(1), other).await.unwrap(),
            StartOutcome::Started(_)
        ));
        assert_eq!(provider.counts.lock().unwrap().creates, 2);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(provider.counts.lock().unwrap().launches, 1);
    }
}

async fn assert_catalog_wait(harness: &Harness, pending: &StartRecord) {
    assert_eq!(
        harness.service.resume(id(1), id(21)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SizeUnavailable)
    );
    assert_eq!(
        harness.snapshot().accounts[0].state.refused_sizes,
        Vec::<String>::new()
    );
    assert_eq!(harness.start(id(21)), *pending);
}
