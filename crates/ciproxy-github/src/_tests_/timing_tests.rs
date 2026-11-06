//! Missing real timestamps keep bounded backoff even after successful lookups.

use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::Duration;
use ciproxy_store_interface::github::{DemandPhase, GithubJob};
use ciproxy_store_interface::github_timing::{JobTimes, PendingTiming};
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::rest::GitHubRestMock;
use crate::runtime::RuntimeMock;
use crate::timing::{JobTiming, RepairTiming, TimingStoreMock, TimingTarget};

use super::support::{at, facts, id};

fn target() -> TimingTarget {
    TimingTarget {
        installation_id: 19,
        repository: "customer/private".into(),
        attempt: PendingTiming {
            attempt_id: id(80),
            job: GithubJob {
                id: id(71),
                organization_id: id(5),
                scale_set_id: id(6),
                label_id: id(2),
                actions_job_id: None,
                request_id: 0,
                run_id: 31,
                repository: "customer/private".into(),
                name: "build".into(),
                workflow_ref: "ci.yml".into(),
                phase: DemandPhase::Completed,
                result: None,
                assigned_at: Some(at() - Duration::minutes(1)),
                started_at: None,
                completed_at: None,
                runner_github_id: Some(71),
                facts: facts(),
            },
        },
    }
}

#[tokio::test]
async fn failed_absent_and_incomplete_times_share_exponential_backoff() {
    let seconds = Arc::new(AtomicI64::new(0));
    let observed_clock = seconds.clone();
    let called_clock = seconds.clone();
    let lookups = Arc::new(Mutex::new(Vec::new()));
    let calls = lookups.clone();
    let saved = Arc::new(Mutex::new(Vec::new()));
    let saves = saved.clone();
    let mock = Arc::new(Unimock::new((
        RuntimeMock::now
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                at() + Duration::seconds(observed_clock.load(Ordering::SeqCst))
            })),
        TimingStoreMock::pending
            .each_call(matching!(_))
            .answers(&|_, _| Ok(vec![target()])),
        TimingStoreMock::save
            .each_call(matching!(_, _, _, _))
            .answers_arc(Arc::new(move |_, workspace, attempt, times, _| {
                assert_eq!((workspace, attempt), (id(1), id(80)));
                saves.lock().unwrap().push(times);
                Ok(())
            })),
        GitHubRestMock::job_times
            .each_call(matching!(19, "customer/private", 31, 71))
            .answers_arc(Arc::new(move |_, _, _, _, _| {
                let second = called_clock.load(Ordering::SeqCst);
                calls.lock().unwrap().push(second);
                match second {
                    0 => Err(Error::AccessUnavailable(503)),
                    30 => Ok(None),
                    _ => Ok(Some(JobTimes {
                        actions_job_id: 333,
                        started_at: (second >= 210).then_some(at()),
                        completed_at: (second >= 750).then_some(at() + Duration::minutes(1)),
                        terminal: true,
                    })),
                }
            })),
    )));
    let repair = RepairTiming::new(mock.clone(), mock.clone(), mock);
    for second in [0, 29, 30, 89, 90, 209, 210, 449, 450, 749, 750] {
        seconds.store(second, Ordering::SeqCst);
        repair.repair(id(1)).await.unwrap();
    }
    assert_eq!(*lookups.lock().unwrap(), vec![0, 30, 90, 210, 450, 750]);
    let saved = saved.lock().unwrap();
    assert_eq!(saved.len(), 4);
    assert_eq!(saved[0].started_at, None);
    assert_eq!(
        saved[3],
        JobTimes {
            actions_job_id: 333,
            started_at: Some(at()),
            completed_at: Some(at() + Duration::minutes(1)),
            terminal: true
        }
    );
}
