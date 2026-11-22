//! Exit claims and retirement results while a workspace session owns an older copy.

use std::sync::Barrier;
use std::thread;

use chrono::Duration;
use ciproxy_store_interface::overflow::admission::{ReservationBudget, ReservationOutcome};
use ciproxy_store_interface::overflow::known::KnownSandbox;
use ciproxy_store_interface::overflow::record::{EncryptedLaunch, StartPhase};
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::runtime_results::{BoatRunnerResult, BoatRunnerWrite};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::{Store, StoreFactory};
use ciproxy_store_pg::factory::PgStoreFactory;
use rust_decimal::Decimal;
use uuid::Uuid;

use super::super::support::{Database, at};
use super::support::{create_runner_intent, reserve, setup};

#[test]
fn callback_claim_is_scoped_one_use_and_durable_before_deletion() {
    let database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let other = database.workspace();
    let store = database.store.as_ref();
    let hash = TokenHash::new(vec![5, 6, 7]);
    create_runner_intent(&database, workspace_id, &record);
    let mut session = store.lock_overflow(workspace_id).unwrap();
    assert_eq!(
        reserve(
            &mut *session,
            &record,
            &EncryptedLaunch(vec![71]),
            &hash,
            &ReservationBudget {
                current_month_usd: Decimal::new(462, 3),
                next_month_usd: None,
            },
        )
        .unwrap(),
        ReservationOutcome::Reserved
    );
    record.sandbox = Some(KnownSandbox::observed(
        "bx_23456789".to_owned(),
        record.starts_at,
        record.dispatch.as_ref().unwrap().lifetime_seconds,
    ));
    record.phase = StartPhase::Running;
    session.save_start(&record).unwrap();
    let runner_id = record.runner_id;
    store
        .update_runner(
            workspace_id,
            record.organization_id,
            runner_id,
            RunnerState::Ready,
            Some(84),
        )
        .unwrap();
    let guard = store
        .boat_runner_guard(workspace_id, runner_id)
        .unwrap()
        .unwrap();
    assert!(matches!(
        store
            .apply_boat_runner_result(workspace_id, &guard, BoatRunnerResult::Checked)
            .unwrap(),
        BoatRunnerWrite::Applied(_)
    ));
    assert_eq!(
        store.overflow_token_workspace(&hash).unwrap(),
        Some(workspace_id)
    );
    assert_eq!(
        store.accept_end(other.id, &hash, runner_id, at()).unwrap(),
        None
    );
    assert_eq!(
        store
            .accept_end(workspace_id, &hash, Uuid::new_v4(), at())
            .unwrap(),
        None
    );

    let reported_at = at() + Duration::hours(7);
    let competitor = PgStoreFactory.connect(&database.url).unwrap();
    let ready = Barrier::new(2);
    let claim = |store: &dyn Store| {
        ready.wait();
        store
            .accept_end(workspace_id, &hash, runner_id, reported_at)
            .unwrap()
    };
    let accepted = thread::scope(|scope| {
        let worker = scope.spawn(|| claim(competitor.as_ref()));
        let first = claim(store);
        let second = worker.join().unwrap();
        assert_eq!(
            usize::from(first.is_some()) + usize::from(second.is_some()),
            1
        );
        first.or(second).unwrap()
    });
    assert_eq!(accepted.phase, StartPhase::Deleting);
    assert_eq!(accepted.reported_at, Some(reported_at));
    assert_eq!(session.snapshot().unwrap().starts, vec![accepted.clone()]);
    assert_eq!(
        store
            .accept_end(workspace_id, &hash, runner_id, at() + Duration::hours(8))
            .unwrap(),
        None
    );
    assert_eq!(
        store.runners(workspace_id, record.organization_id).unwrap()[0]
            .size
            .disk_bytes,
        111_000_000_000
    );

    let claimed_guard = store
        .boat_runner_guard(workspace_id, runner_id)
        .unwrap()
        .unwrap();
    assert_eq!(guard.runner, claimed_guard.runner);
    assert_eq!(
        claimed_guard.delivery.as_ref().unwrap().reported_at,
        Some(reported_at)
    );
    assert!(matches!(
        store
            .apply_boat_runner_result(workspace_id, &guard, BoatRunnerResult::Checked)
            .unwrap(),
        BoatRunnerWrite::Rejected
    ));
    assert!(matches!(
        store
            .apply_boat_runner_result(workspace_id, &claimed_guard, BoatRunnerResult::Checked)
            .unwrap(),
        BoatRunnerWrite::Applied(_)
    ));

    session.save_start(&record).unwrap();
    let saved = session.snapshot().unwrap().starts.remove(0);
    assert_eq!(saved.reported_at, Some(reported_at));
    assert_eq!(saved.phase, StartPhase::Deleting);
    assert_eq!(saved.ends_at, None);

    let first_end = at() + Duration::hours(9);
    record.phase = StartPhase::Ended;
    record.ends_at = Some(first_end);
    session.save_start(&record).unwrap();
    let ended = session.snapshot().unwrap().starts.remove(0);
    assert_eq!(ended.reported_at, Some(reported_at));
    assert_eq!(ended.phase, StartPhase::Ended);
    assert_eq!(ended.ends_at, Some(first_end));

    session.save_start(&accepted).unwrap();
    assert_eq!(session.snapshot().unwrap().starts, vec![ended.clone()]);

    record.reported_at = Some(at() + Duration::hours(8));
    for stale_end in [
        first_end + Duration::minutes(1),
        first_end - Duration::minutes(1),
    ] {
        record.ends_at = Some(stale_end);
        session.save_start(&record).unwrap();
        assert_eq!(session.snapshot().unwrap().starts, vec![ended.clone()]);
    }
    assert_eq!(
        store
            .accept_end(
                workspace_id,
                &hash,
                runner_id,
                first_end + Duration::minutes(2)
            )
            .unwrap(),
        None
    );
}
