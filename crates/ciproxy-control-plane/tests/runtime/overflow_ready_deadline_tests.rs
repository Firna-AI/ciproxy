//! A Boat runner that takes no waiting job gives way at its ready deadline.

use std::sync::atomic::Ordering;

use chrono::{DateTime, Utc};
use ciproxy_store_interface::overflow::record::StartPhase;

use super::overflow_support::Harness;
use super::settled::Settled;

#[tokio::test]
async fn overflow_replaces_a_launched_runner_at_its_ready_deadline() {
    let harness = Harness::new(false, false).await;
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

    *harness.ports.now.lock().unwrap() = deadline;
    harness.runtime.settled_tick().await.unwrap();
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
        .boat_runtime_snapshot(harness.fixture.workspace)
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
