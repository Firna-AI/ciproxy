//! Unsent reservations need no provider key when demand or key authority ends.

use std::sync::{Arc, Mutex};

use ciproxy_overflow::api::{Overflow, Retirement, RunnerAuthorityMock, StartOutcome};
use ciproxy_overflow::error::Error;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use unimock::{MockFn, matching};

use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, record, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn cleanup_cancels_unneeded_unsent_and_keeps_needed_reservations() {
    let provider = Provider::new(DisableAt::Never);
    let retirement = Arc::new(Mutex::new(Retirement::WaitingDemand));
    let retiring = retirement.clone();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let retired = calls.clone();
    let runners = RunnerAuthorityMock::retire
        .some_call(matching!(_, _, _))
        .answers_arc(Arc::new(move |_, workspace, organization, runner| {
            retired
                .lock()
                .unwrap()
                .push((workspace, organization, runner));
            Ok(*retiring.lock().unwrap())
        }));
    let harness = Harness::new(provider.backend, runners, Runtime::Cancellation);
    let mut pending = record(StartPhase::Unsent);
    pending.sandbox = None;
    pending.start_usd = None;
    harness.seed(pending);
    harness.service.cleanup(id(1)).await.unwrap();
    assert_eq!(*calls.lock().unwrap(), [(id(1), id(3), id(12))]);
    assert_eq!(harness.start(id(11)).phase, StartPhase::Unsent);
    assert_ne!(
        harness.database.session().encrypted_launch(id(11)).unwrap(),
        None
    );
    *retirement.lock().unwrap() = Retirement::Removed;
    harness.service.cleanup(id(1)).await.unwrap();
    assert_eq!(
        *calls.lock().unwrap(),
        [(id(1), id(3), id(12)), (id(1), id(3), id(12))]
    );
    let cancelled = harness.start(id(11));
    assert_eq!(cancelled.phase, StartPhase::CancelledUnsent);
    assert_eq!(cancelled.sandbox, None);
    assert_eq!(cancelled.billing, None);
    assert_eq!(cancelled.start_usd, None);
    assert_eq!(
        harness.database.session().encrypted_launch(id(11)).unwrap(),
        None
    );
    assert_eq!(harness.state.lock().unwrap().key_opens, 0);
    assert_eq!(provider.counts.lock().unwrap().creates, 0);
    assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
    assert_eq!(provider.counts.lock().unwrap().deletes, 0);
}

#[tokio::test]
async fn replacement_cancels_unsent_without_opening_the_old_key() {
    for key_open in [2, 3] {
        let provider = Provider::new(DisableAt::Never);
        let harness = Harness::new(provider.backend, (), Runtime::Start);
        harness.state.lock().unwrap().fail_on_key_open = Some(key_open);
        assert!(matches!(
            harness.service.start(id(1), request()).await,
            Err(Error::KmsHttp { status: 503 })
        ));
        let opens = harness.state.lock().unwrap().key_opens;
        harness
            .service
            .connect(
                id(1),
                id(44),
                Secret::new("replacement-key"),
                "personal".to_owned(),
                200_000_000,
            )
            .await
            .unwrap();
        assert_eq!(harness.start(id(11)).phase, StartPhase::CancelledUnsent);
        assert_eq!(
            harness
                .database
                .store
                .encrypted_boat_key(id(1), id(2))
                .unwrap(),
            None
        );
        assert_eq!(harness.snapshot().accounts.len(), 1);
        assert_eq!(harness.snapshot().accounts[0].account.id, id(44));
        assert_eq!(harness.state.lock().unwrap().key_opens, opens);
        assert!(matches!(
            harness.service.resume(id(1), id(11)).await.unwrap(),
            StartOutcome::Ended(_)
        ));
        let view = harness.service.view(id(1)).await.unwrap();
        assert_eq!(view.spend.counted_usd().unwrap(), Decimal::ZERO);
        assert_eq!(provider.counts.lock().unwrap().creates, 0);
        assert_eq!(provider.counts.lock().unwrap().recoveries, 0);
        assert_eq!(provider.counts.lock().unwrap().deletes, 0);
    }
}
