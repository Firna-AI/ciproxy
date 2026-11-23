//! Exit claims and retirement results while a workspace session owns an older copy.

use std::thread;

use chrono::Duration;
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use ciproxy_store_interface::runtime_results::{OverflowRunnerResult, OverflowRunnerWrite};
use ciproxy_store_interface::secrets::TokenHash;
use ciproxy_store_interface::store::Store;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use uuid::Uuid;

use crate::store::metering::concurrent::support::{tagged_store, wait_for_blocker};

use super::super::support::{Database, at};
use super::support::{create_runner_intent, hold_delivery_row, save_running_delivery, setup};

#[test]
fn callback_claim_is_scoped_one_use_and_durable_before_deletion() {
    let mut database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    let other = database.workspace();
    let store = database.store.as_ref();
    let hash = TokenHash::new(vec![5, 6, 7]);
    create_runner_intent(&database, workspace_id, &record);
    let mut session = store.lock_overflow(workspace_id).unwrap();
    save_running_delivery(&mut *session, &mut record, &hash);
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
        .overflow_runner_guard(workspace_id, runner_id)
        .unwrap()
        .unwrap();
    assert!(matches!(
        store
            .apply_overflow_runner_result(workspace_id, &guard, OverflowRunnerResult::Checked)
            .unwrap(),
        OverflowRunnerWrite::Applied(_)
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
    let first_claim = tagged_store(&database.url);
    let second_claim = tagged_store(&database.url);
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let controller_pid = hold_delivery_row(&mut database.connection, record.delivery_id);
    let claim = |store: &dyn Store| {
        store
            .accept_end(workspace_id, &hash, runner_id, reported_at)
            .unwrap()
    };
    let accepted = thread::scope(|scope| {
        let first = scope.spawn(|| claim(first_claim.store.as_ref()));
        wait_for_blocker(&mut observer, first_claim.pid, controller_pid);
        let second = scope.spawn(|| claim(second_claim.store.as_ref()));
        wait_for_blocker(&mut observer, second_claim.pid, first_claim.pid);
        database.connection.batch_execute("COMMIT").unwrap();
        let first = first.join().unwrap();
        let second = second.join().unwrap();
        assert_eq!(
            usize::from(first.is_some()) + usize::from(second.is_some()),
            1,
            "only one overlapping exit claim may succeed"
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
        .overflow_runner_guard(workspace_id, runner_id)
        .unwrap()
        .unwrap();
    assert_eq!(guard.runner, claimed_guard.runner);
    assert_eq!(
        claimed_guard.delivery.as_ref().unwrap().reported_at,
        Some(reported_at)
    );
    assert!(matches!(
        store
            .apply_overflow_runner_result(workspace_id, &guard, OverflowRunnerResult::Checked)
            .unwrap(),
        OverflowRunnerWrite::Rejected
    ));
    assert!(matches!(
        store
            .apply_overflow_runner_result(
                workspace_id,
                &claimed_guard,
                OverflowRunnerResult::Checked
            )
            .unwrap(),
        OverflowRunnerWrite::Applied(_)
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
        first_end - Duration::minutes(1),
        first_end + Duration::minutes(1),
    ] {
        record.ends_at = Some(stale_end);
        session.save_start(&record).unwrap();
        assert_eq!(session.snapshot().unwrap().starts, vec![ended.clone()]);
        session.collect(first_end - Duration::seconds(1)).unwrap();
        assert_eq!(
            session.snapshot().unwrap().starts,
            vec![ended.clone()],
            "a cutoff before the first end must keep the delivery after saving {stale_end}"
        );
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

    session.collect(first_end + Duration::seconds(1)).unwrap();
    assert_eq!(
        session.snapshot().unwrap().starts,
        [],
        "a cutoff after the first end must remove the delivery"
    );
}

#[test]
fn stale_delivery_save_waits_for_claim_and_preserves_it() {
    let mut database = Database::new();
    let (workspace_id, _, mut record) = setup(&database);
    create_runner_intent(&database, workspace_id, &record);
    let saver = tagged_store(&database.url);
    let mut session = saver.store.lock_overflow(workspace_id).unwrap();
    let hash = TokenHash::new(vec![5, 6, 7]);
    save_running_delivery(&mut *session, &mut record, &hash);
    let stale = session.snapshot().unwrap().starts.remove(0);
    assert_eq!(
        database
            .store
            .runners(workspace_id, stale.organization_id)
            .unwrap()
            .into_iter()
            .map(|runner| (runner.id, runner.state))
            .collect::<Vec<_>>(),
        vec![(stale.runner_id, RunnerState::Starting)],
        "the claim must meet a starting runner"
    );
    let mut observer = PgConnection::establish(&database.url).unwrap();
    let saver_pid = saver.workspace_session_pid(&mut observer);
    let claimant = tagged_store(&database.url);
    let controller_pid = hold_delivery_row(&mut database.connection, stale.delivery_id);
    let reported_at = at() + Duration::hours(7);
    let accepted = thread::scope(|scope| {
        let claim = scope.spawn(|| {
            claimant
                .store
                .accept_end(workspace_id, &hash, stale.runner_id, reported_at)
        });
        wait_for_blocker(&mut observer, claimant.pid, controller_pid);
        let save = scope.spawn(|| session.save_start(&stale));
        wait_for_blocker(&mut observer, saver_pid, claimant.pid);
        database.connection.batch_execute("COMMIT").unwrap();
        let accepted = claim.join().unwrap().unwrap();
        save.join().unwrap().unwrap();
        accepted
    });

    assert_eq!(accepted.unwrap().reported_at, Some(reported_at));
    let saved = session.snapshot().unwrap().starts.remove(0);
    assert_eq!(
        saved.reported_at,
        Some(reported_at),
        "the stale write must retain the concurrent claim"
    );
    assert_eq!(saved.phase, StartPhase::Deleting);
    assert_eq!(
        database
            .store
            .accept_end(
                workspace_id,
                &hash,
                stale.runner_id,
                at() + Duration::hours(8)
            )
            .unwrap(),
        None,
        "the stale write must not let the token claim again"
    );
    assert_eq!(
        database
            .store
            .runners(workspace_id, stale.organization_id)
            .unwrap()
            .into_iter()
            .map(|runner| (runner.id, runner.state))
            .collect::<Vec<_>>(),
        vec![(stale.runner_id, RunnerState::Stopping)],
        "the stale write must keep the runner stopping after the claim"
    );
}
