//! Lost launch responses and running guests keep only recovery authority.

use ciproxy_overflow::api::{Overflow, Retirement, RunnerAuthorityMock, StartOutcome};
use ciproxy_overflow::error::Error;
use ciproxy_overflow::maintenance::LaunchState;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::StartPhase;
use unimock::{MockFn, matching};

use super::authority_actions::{Entry, Withdrawal, reconcile, withdraw};
use super::authority_support::{DisableAt, Provider};
use super::fixtures::{id, record, request};
use super::harness::{Harness, Runtime};

#[tokio::test]
async fn retirement_preserves_an_already_running_runner_until_its_end() {
    for withdrawal in [Withdrawal::Disconnect, Withdrawal::Replacement] {
        let provider = Provider::new(DisableAt::Never);
        let retirement = RunnerAuthorityMock::retire
            .some_call(matching!(_, _, _))
            .returns(Ok(Retirement::RunningJob));
        let kind = match withdrawal {
            Withdrawal::Replacement => Runtime::Authority,
            _ => Runtime::Read,
        };
        let harness = Harness::new(provider.backend, retirement, kind);
        harness.seed(record(StartPhase::Running));
        withdraw(&harness, withdrawal).await;
        let resumed = harness.service.resume(id(1), id(11)).await.unwrap();
        assert!(matches!(resumed, StartOutcome::Started(_)));
        reconcile(&harness, Entry::Duplicate).await;
        reconcile(&harness, Entry::Cleanup).await;
        {
            let counts = provider.counts.lock().unwrap();
            assert_eq!(counts.inputs, 0);
            assert_eq!(counts.launches, 0);
            assert_eq!(counts.deletes, 0);
        }
        assert_eq!(harness.start(id(11)).phase, StartPhase::Running);
        assert_eq!(
            harness
                .database
                .store
                .encrypted_boat_key(id(1), id(2))
                .unwrap()
                .unwrap()
                .as_bytes(),
            &[7, 7]
        );
        let token = Secret::new(format!("seed-callback-{}", id(11)));
        harness.service.report_end(token, id(12)).await.unwrap();
        assert_eq!(provider.counts.lock().unwrap().deletes, 1);
        assert_eq!(harness.start(id(11)).phase, StartPhase::Ended);
        assert_eq!(
            harness
                .database
                .store
                .encrypted_boat_key(id(1), id(2))
                .unwrap(),
            None
        );
    }
}

#[tokio::test]
async fn retired_launching_delivery_inspects_the_marker_and_preserves_a_started_guest() {
    for withdrawal in [Withdrawal::Disconnect, Withdrawal::Replacement] {
        let provider = Provider::new(DisableAt::Never);
        *provider.launched.lock().unwrap() = Some(LaunchState::Started);
        let retirement = RunnerAuthorityMock::retire
            .some_call(matching!(_, _, _))
            .returns(Ok(Retirement::RunningJob));
        let kind = match withdrawal {
            Withdrawal::Replacement => Runtime::Authority,
            _ => Runtime::Read,
        };
        let harness = Harness::new(provider.backend, retirement, kind);
        harness.seed(record(StartPhase::Launching));
        withdraw(&harness, withdrawal).await;
        harness.state.lock().unwrap().active_backends = 0;
        for entry in [Entry::Resume, Entry::Duplicate, Entry::Cleanup] {
            reconcile(&harness, entry).await;
            assert_eq!(harness.start(id(11)).phase, StartPhase::Running);
            let counts = provider.counts.lock().unwrap();
            assert_eq!(counts.inputs, 0);
            assert_eq!(counts.launches, 0);
            assert_eq!(counts.deletes, 0);
        }
        assert_eq!(harness.state.lock().unwrap().active_backends, 0);
        assert_eq!(
            harness.database.session().encrypted_launch(id(11)).unwrap(),
            None
        );
        assert!(
            harness
                .database
                .store
                .encrypted_boat_key(id(1), id(2))
                .unwrap()
                .is_some()
        );
    }
}

#[tokio::test]
async fn retired_launching_delivery_keeps_recovery_authority_when_inspection_is_unavailable() {
    let provider = Provider::new(DisableAt::Never);
    *provider.launched.lock().unwrap() = None;
    let harness = Harness::new(provider.backend, (), Runtime::Read);
    harness.seed(record(StartPhase::Launching));
    harness.service.disconnect(id(1)).await.unwrap();
    let result = harness.service.resume(id(1), id(11)).await.unwrap();
    assert!(matches!(result, StartOutcome::Pending(_)));
    assert!(matches!(
        harness.service.cleanup(id(1)).await,
        Err(Error::ProviderUnavailable)
    ));
    assert_eq!(harness.start(id(11)).phase, StartPhase::Launching);
    assert_eq!(harness.start(id(11)).ends_at, None);
    assert!(
        harness
            .database
            .session()
            .encrypted_launch(id(11))
            .unwrap()
            .is_some()
    );
    assert!(
        harness
            .database
            .store
            .encrypted_boat_key(id(1), id(2))
            .unwrap()
            .is_some()
    );
    let counts = provider.counts.lock().unwrap();
    assert_eq!(counts.inputs, 0);
    assert_eq!(counts.launches, 0);
    assert_eq!(counts.deletes, 0);
}

#[tokio::test]
async fn retired_ambiguous_create_stays_recover_only_until_its_identity_is_found() {
    let provider = Provider::new(DisableAt::Never);
    *provider.recovered.lock().unwrap() = false;
    let harness = Harness::new(provider.backend, (), Runtime::Read);
    let mut creating = record(StartPhase::Creating);
    creating.sandbox = None;
    harness.seed(creating);
    harness.service.disconnect(id(1)).await.unwrap();
    for entry in [Entry::Resume, Entry::Duplicate, Entry::Cleanup] {
        reconcile(&harness, entry).await;
        assert_eq!(harness.start(id(11)).phase, StartPhase::Creating);
        assert_eq!(harness.start(id(11)).ends_at, None);
        assert_eq!(harness.start(id(11)).sandbox, None);
    }
    {
        let counts = provider.counts.lock().unwrap();
        assert_eq!(counts.creates, 0);
        assert_eq!(counts.inputs, 0);
        assert_eq!(counts.launches, 0);
        assert_eq!(counts.deletes, 0);
    }
    assert!(
        harness
            .database
            .store
            .encrypted_boat_key(id(1), id(2))
            .unwrap()
            .is_some()
    );
    *provider.recovered.lock().unwrap() = true;
    let recovered = harness.service.resume(id(1), id(11)).await.unwrap();
    assert!(matches!(recovered, StartOutcome::Ended(_)));
    assert_eq!(provider.counts.lock().unwrap().deletes, 1);
    assert_eq!(provider.counts.lock().unwrap().launches, 0);
    harness.service.cleanup(id(1)).await.unwrap();
    assert_eq!(
        harness
            .database
            .store
            .encrypted_boat_key(id(1), id(2))
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn replacement_allows_only_its_active_key_to_start_a_new_runner() {
    let provider = Provider::new(DisableAt::Never);
    let harness = Harness::new(provider.backend, (), Runtime::Start);
    harness.seed(record(StartPhase::Preparing));
    withdraw(&harness, Withdrawal::Replacement).await;
    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    let result = harness.service.start(id(1), next).await.unwrap();
    let StartOutcome::Started(started) = result else {
        panic!("replacement start must launch");
    };
    assert_eq!(started.key_id, id(44));
    assert_eq!(started.runner_id, id(22));
    assert_eq!(harness.start(id(11)).phase, StartPhase::Preparing);
    assert!(matches!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Ended(_)
    ));
    let counts = provider.counts.lock().unwrap();
    assert_eq!(counts.creates, 1);
    assert_eq!(counts.inputs, 1);
    assert_eq!(counts.launches, 1);
    assert_eq!(counts.deletes, 1);
}
