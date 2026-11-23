//! Proven local create rejections release demand, costs, and retained credentials.

use ciproxy_overflow::api::{Overflow, StartOutcome};
use ciproxy_overflow::error::Error;
use ciproxy_overflow::provider::BoatAccountApiMock;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::overflow::record::StartPhase;
use rust_decimal::Decimal;
use sandbox_interface::SandboxLifecycleMock;
use unimock::{MockFn, Unimock, matching};

use super::super::fixtures::{at, capabilities, facts, id, request, snapshot};
use super::super::harness::{Harness, Runtime};

#[tokio::test]
async fn unsent_create_ends_without_cost_or_authority_and_permits_a_new_delivery() {
    let backend = capabilities(Unimock::new((
        BoatAccountApiMock::account
            .some_call(matching!())
            .answers(&|_| Ok(facts())),
        SandboxLifecycleMock::create_sandbox
            .some_call(matching!(_))
            .answers(&|_, _| {
                Err(sandbox_interface::Error::SandboxCreateNotSent {
                    source: Box::new(sandbox_interface::Error::UnknownProfile),
                })
            })
            .n_times(2),
    )));
    let harness = Harness::new(backend, (), Runtime::Start);
    let outcome = harness.service.start(id(1), request()).await.unwrap();
    let ended = harness.start(id(11));
    assert_eq!(ended.phase, StartPhase::Refused);
    assert_eq!(outcome, StartOutcome::Ended(ended.clone()));
    assert_eq!(ended.sandbox, None);
    assert_eq!(ended.ends_at, Some(at()));
    assert_eq!(ended.start_usd, None);
    assert!(ended.runtime_complete);
    assert_eq!(
        harness.database.session().encrypted_launch(id(11)).unwrap(),
        None
    );
    assert!(matches!(
        harness
            .service
            .report_end(Secret::new("callback-token-for-one-sandbox"), id(12), None)
            .await,
        Err(Error::InvalidToken)
    ));
    let view = harness.service.view(id(1)).await.unwrap();
    assert_eq!(view.spend.counted_usd().unwrap(), Decimal::ZERO);
    assert!(view.spend.complete());
    assert_eq!(view.state, snapshot().accounts[0].state);
    assert_eq!(
        harness.service.resume(id(1), id(11)).await.unwrap(),
        StartOutcome::Ended(ended)
    );

    let mut next = request();
    next.delivery_id = id(21);
    next.runner_id = id(22);
    let next_outcome = harness.service.start(id(1), next).await.unwrap();
    let next_ended = harness.start(id(21));
    assert_eq!(next_ended.phase, StartPhase::Refused);
    assert_eq!(next_outcome, StartOutcome::Ended(next_ended));
    assert_eq!(
        harness.state.lock().unwrap().phases,
        [
            StartPhase::Unsent,
            StartPhase::Creating,
            StartPhase::Refused,
            StartPhase::Unsent,
            StartPhase::Creating,
            StartPhase::Refused,
        ]
    );
    harness.service.disconnect(id(1)).await.unwrap();
    assert_eq!(
        harness
            .database
            .store
            .encrypted_boat_key(id(1), id(2))
            .unwrap(),
        None
    );
}
