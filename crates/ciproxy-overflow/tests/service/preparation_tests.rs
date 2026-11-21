//! Proven unsent requests retry preparation and release custody without provider facts.

use chrono::{DateTime, Utc};
use ciproxy_overflow::api::{Overflow, StartOutcome, WaitReason};
use ciproxy_overflow::error::Error;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn key_preparation_failures_retry_the_same_delivery_once() {
    for key_open in [2, 3] {
        let provider = Provider::new(DisableAt::Never);
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        harness.state.lock().unwrap().fail_on_key_open = Some(key_open);
        assert!(matches!(
            harness.service.start(id(1), request()).await,
            Err(Error::KmsHttp { status: 503 })
        ));
        assert_eq!(harness.start(id(11)).phase, StartPhase::Unsent);
        assert_eq!(provider.counts.lock().unwrap().creates, 0);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(harness.snapshot().starts.len(), 1);

        let retry_at = "2026-12-01T00:00:00Z".parse().unwrap();
        *harness.now.lock().unwrap() = retry_at;
        let result = harness.service.start(id(1), request()).await.unwrap();
        assert!(matches!(result, StartOutcome::Started(_)));
        let saved = harness.start(id(11));
        assert_eq!(saved.starts_at, retry_at);
        assert_eq!(
            saved.sandbox.unwrap().lifetime_ends_by(),
            "2026-12-01T06:00:00Z".parse::<DateTime<Utc>>().unwrap()
        );
        assert_eq!(saved.start_usd, None);
        assert_eq!(provider.counts.lock().unwrap().creates, 1);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert!(matches!(
            harness.service.resume(id(1), id(11)).await.unwrap(),
            StartOutcome::Started(_)
        ));
        assert_eq!(provider.counts.lock().unwrap().creates, 1);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(provider.counts.lock().unwrap().launches, 1);
        assert_eq!(harness.snapshot().starts.len(), 1);
        let view = harness.service.view(id(1)).await.unwrap();
        assert_eq!(view.spend.counted_usd().unwrap(), Decimal::new(462, 3));
    }
}

#[tokio::test]
async fn disconnect_cancels_unsent_without_keys_or_provider_access() {
    for key_open in [2, 3] {
        let provider = Provider::new(DisableAt::Never);
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        harness.state.lock().unwrap().fail_on_key_open = Some(key_open);
        assert!(matches!(
            harness.service.start(id(1), request()).await,
            Err(Error::KmsHttp { status: 503 })
        ));
        harness.service.disconnect(id(1)).await.unwrap();
        let opens = harness.state.lock().unwrap().key_opens;
        assert!(matches!(
            harness.service.resume(id(1), id(11)).await.unwrap(),
            StartOutcome::Ended(_)
        ));
        assert!(matches!(
            harness.service.start(id(1), request()).await.unwrap(),
            StartOutcome::Ended(_)
        ));
        harness.service.cleanup(id(1)).await.unwrap();
        let cancelled = harness.start(id(11));
        assert_eq!(cancelled.phase, StartPhase::CancelledUnsent);
        assert_eq!(cancelled.sandbox, None);
        assert_eq!(cancelled.start_usd, None);
        assert_eq!(cancelled.billing, None);
        assert_eq!(cancelled.ends_at, Some(*harness.now.lock().unwrap()));
        assert_eq!(harness.snapshot().accounts, []);
        assert_eq!(
            harness.database.session().encrypted_launch(id(11)).unwrap(),
            None
        );
        let token = ciproxy_protocol::secret::Secret::new("callback-token-for-one-sandbox");
        assert!(matches!(
            harness.service.report_end(token, id(12)).await,
            Err(Error::InvalidToken)
        ));
        let view = harness.service.view(id(1)).await.unwrap();
        assert_eq!(view.spend.counted_usd().unwrap(), Decimal::ZERO);
        assert_eq!(harness.state.lock().unwrap().key_opens, opens);
        let counts = provider.counts.lock().unwrap();
        assert_eq!(counts.creates, 0);
        assert_eq!(counts.recoveries, 0);
        assert_eq!(counts.deletes, 0);
    }
}

#[tokio::test]
async fn unsent_retry_rechecks_current_limit_before_dispatch() {
    let provider = Provider::new(DisableAt::Never);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    harness.state.lock().unwrap().fail_on_key_open = Some(3);
    assert!(matches!(
        harness.service.start(id(1), request()).await,
        Err(Error::KmsHttp { status: 503 })
    ));
    *harness.now.lock().unwrap() = "2026-12-01T00:00:00Z".parse().unwrap();
    harness.set_limit(100_000);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Waiting(WaitReason::SpendLimit {
            resumes_at: "2027-01-01T00:00:00Z".parse().unwrap(),
        })
    );
    assert_eq!(harness.start(id(11)).phase, StartPhase::Unsent);
    assert_eq!(provider.counts.lock().unwrap().creates, 0);
    assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
    harness.set_limit(200_000_000);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Started(_)
    ));
    assert_eq!(provider.counts.lock().unwrap().creates, 1);
}
