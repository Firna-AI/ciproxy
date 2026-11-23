//! A Boat runner that takes no waiting job gives way at its ready deadline.

use std::sync::atomic::Ordering;
use std::sync::{Arc, OnceLock};

use chrono::{DateTime, Utc};
use ciproxy_github::registrations::{
    RegisteredRunner, RegistrationRetirement, RunnerRegistrationsMock,
};
use ciproxy_store_interface::overflow::record::StartPhase;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use super::overflow_support::Harness;
use super::settled::Settled;

#[tokio::test]
async fn overflow_replaces_a_launched_runner_at_its_ready_deadline() {
    let mut harness = Harness::new(false, false).await;
    harness.queued();
    let launch = "2026-11-05T12:00:00Z".parse().unwrap();
    let deadline = "2026-11-05T12:10:00Z".parse().unwrap();
    let one_launch = Progress {
        creates: 1,
        deletes: 0,
        launches: 1,
        starts: vec![(StartPhase::Running, Some(launch), None)],
    };

    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(progress(&harness), one_launch, "after the first launch");

    *harness.ports.now.lock().unwrap() = "2026-11-05T12:09:59Z".parse().unwrap();
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(
        progress(&harness),
        one_launch,
        "one second before the deadline"
    );

    let first_removal = capture_deletes_at_first_github_removal(&mut harness);
    *harness.ports.now.lock().unwrap() = deadline;
    harness.runtime.settled_tick().await.unwrap();
    assert_eq!(
        first_removal.get().copied(),
        Some(0),
        "sandbox deletes at the first GitHub registration removal"
    );
    assert_eq!(
        progress(&harness),
        Progress {
            creates: 2,
            deletes: 1,
            launches: 2,
            starts: vec![
                (StartPhase::Ended, Some(launch), Some(deadline)),
                (StartPhase::Running, Some(deadline), None),
            ],
        },
        "at the deadline"
    );
}

#[tokio::test]
async fn overflow_ready_deadline_keeps_a_runner_that_github_reports_busy() {
    let harness = Harness::new(false, false).await;
    harness.queued();
    harness.runtime.settled_tick().await.unwrap();

    let launch = "2026-11-05T12:00:00Z".parse().unwrap();
    let deadline = "2026-11-05T12:10:00Z".parse().unwrap();
    harness.busy.store(true, Ordering::SeqCst);
    *harness.ports.now.lock().unwrap() = deadline;
    harness.runtime.settled_tick().await.unwrap();

    let old_runner_state = harness
        .fixture
        .store
        .runners(harness.fixture.workspace, harness.fixture.organization)
        .unwrap()
        .into_iter()
        .find(|runner| runner.github_id == Some(100))
        .map(|runner| runner.state);
    assert_eq!(
        (progress(&harness), old_runner_state),
        (
            Progress {
                creates: 2,
                deletes: 0,
                launches: 2,
                starts: vec![
                    (StartPhase::Running, Some(launch), None),
                    (StartPhase::Running, Some(deadline), None),
                ],
            },
            Some(RunnerState::Busy),
        ),
        "GitHub Busy preserves the old runner while the replacement launches"
    );
}

fn capture_deletes_at_first_github_removal(harness: &mut Harness) -> Arc<OnceLock<usize>> {
    let first_removal = Arc::new(OnceLock::new());
    let recorded = first_removal.clone();
    let ports = harness.ports.clone();
    harness.use_registrations(Arc::new(Unimock::new((
        RunnerRegistrationsMock::create
            .some_call(matching!(_, _))
            .answers(&|_, request, _| {
                Ok(RegisteredRunner {
                    target: request.target,
                    runner_id: 101,
                    configuration: "fake-jit".into(),
                })
            }),
        RunnerRegistrationsMock::retire
            .each_call(matching!(_, _, Some(100)))
            .answers_arc(Arc::new(move |_, _, _, _| {
                recorded.get_or_init(|| ports.deletes.load(Ordering::SeqCst));
                Ok(RegistrationRetirement::Removed { id: 100 })
            })),
    ))));
    first_removal
}

type StartStatus = (StartPhase, Option<DateTime<Utc>>, Option<DateTime<Utc>>);

#[derive(Debug, Eq, PartialEq)]
struct Progress {
    creates: usize,
    deletes: usize,
    launches: usize,
    starts: Vec<StartStatus>,
}

fn progress(harness: &Harness) -> Progress {
    let mut starts = harness
        .fixture
        .store
        .overflow_runtime_snapshot(harness.fixture.workspace)
        .unwrap()
        .starts;
    starts.sort_by_key(|start| start.launched_at);
    Progress {
        creates: harness.ports.creates.lock().unwrap().len(),
        deletes: harness.ports.deletes.load(Ordering::SeqCst),
        launches: harness.ports.launches.load(Ordering::SeqCst),
        starts: starts
            .into_iter()
            .map(|start| (start.phase, start.launched_at, start.ends_at))
            .collect(),
    }
}
