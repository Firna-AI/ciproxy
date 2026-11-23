//! Usage collection and delivery across the supervisor's runner lifecycle.

use std::collections::VecDeque;
use std::future::pending;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::usage::JobUsage;
use tokio::time::{Instant, sleep_until};
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{RunnerEvent, RunnerMock};
use crate::error::Error;
use crate::service::Guest;
use crate::usage::worker_log::LogPagesMock;
use crate::usage::{CpuReading, Reading, ReporterMock, SourceMock, Usage};

use self::usage_steps::{BOAT_LOG, boat_steps, worker_pages};
use super::support::{Observation, Trace, channel, configure, configured, hello, supervisor};

#[path = "usage_steps_tests.rs"]
mod usage_steps;

const EPOCH_MS: i64 = 1_700_000_000_000;

#[tokio::test(start_paused = true)]
async fn handoff_starts_sampling_once_and_a_failed_read_keeps_the_exit_report() {
    let clock = Instant::now();
    let trace = Trace::default();
    let reads = Arc::new(Mutex::new(Vec::new()));
    let job_id = JobId("job-1".into());
    let runtime = RuntimeCredentials {
        docker_data_bytes: 0,
        proxy_url: "http://172.21.0.1:8230/".into(),
        github_token: Secret::new("example-token"),
        cache_url: Secret::new("https://cache.example/"),
        results_url: Secret::new("https://results.example/"),
        permissions: CachePermissions {
            read: true,
            write: false,
        },
    };
    let handoff = RunnerEvent::Starting {
        job_id: job_id.clone(),
        runtime: runtime.clone(),
    };
    let process = runner_at(
        clock,
        &trace,
        vec![
            (10_000, handoff.clone()),
            (11_500, handoff),
            (13_500, RunnerEvent::Exited(RunnerExit { code: Some(7) })),
        ],
    );
    let control = channel(vec![Ok(Some(configure())), Ok(None)], &trace, false);
    let mut guest = supervisor(control, process, &trace);
    guest.usage = Some(Usage::new(
        Box::new(source(clock, &reads, Some(12), None)),
        None,
        Box::new(Unimock::new(
            LogPagesMock::list
                .each_call(matching!())
                .answers(&|_| Ok(Vec::new())),
        )),
    ));

    guest.run().await.unwrap();

    assert_eq!(*reads.lock().unwrap(), [10, 11, 12, 13]);
    let starting = GuestToAgent::JobStarting {
        runner_id: RunnerId("runner-7".into()),
        job_id,
        runtime,
    };
    assert_eq!(
        *trace.lock().unwrap(),
        [
            hello(),
            configured(),
            Observation::Start,
            Observation::ControlLost,
            Observation::Message(starting.clone()),
            Observation::Message(starting),
            exit_with(7, usage(10_000, vec![139], vec![2])),
            Observation::Poweroff,
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn boat_posts_whole_series_and_reports_exit_despite_failed_and_pending_posts() {
    let clock = Instant::now();
    let trace = Trace::default();
    let reads = Arc::new(Mutex::new(Vec::new()));
    let posted = trace.clone();
    let reporter = Unimock::new(
        ReporterMock::post_samples
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, message| {
                let posted = posted.clone();
                Box::pin(async move {
                    posted.lock().unwrap().push(Observation::Message(message));
                    if clock.elapsed().as_secs() == 35 {
                        Err(Error::internal_message("callback refused samples"))
                    } else {
                        pending().await
                    }
                })
            })),
    );
    let process = runner_at(
        clock,
        &trace,
        vec![
            (1_500, RunnerEvent::Ready),
            (75_500, RunnerEvent::Exited(RunnerExit { code: Some(9) })),
        ],
    );
    let control = channel(vec![Ok(Some(configure())), Ok(None)], &trace, false);
    let mut guest = supervisor(control, process, &trace);
    guest.usage = Some(Usage::new(
        Box::new(source(clock, &reads, None, Some(5))),
        Some(Arc::new(reporter)),
        Box::new(worker_pages(clock, |second| {
            let end = if second < 40 {
                BOAT_LOG.find("[2023-11-14 22:14:00Z").unwrap()
            } else {
                BOAT_LOG.len()
            };
            Some(&BOAT_LOG[..end])
        })),
    ));

    guest.run().await.unwrap();

    assert_eq!(*reads.lock().unwrap(), (5..=75).collect::<Vec<_>>());
    let mut final_usage = usage(5_000, (134..=203).collect(), kills(70));
    final_usage.steps = boat_steps(Some(1_700_000_040_000));
    assert_eq!(
        *trace.lock().unwrap(),
        [
            hello(),
            configured(),
            Observation::Start,
            Observation::ControlLost,
            Observation::Message(GuestToAgent::RunnerReady {
                runner_id: RunnerId("runner-7".into())
            }),
            samples_with(163, 30, None),
            samples_with(193, 60, Some(1_700_000_040_000)),
            exit_with(9, final_usage),
            Observation::Poweroff,
        ]
    );
}

fn source(
    clock: Instant,
    reads: &Arc<Mutex<Vec<u64>>>,
    fail_at: Option<u64>,
    worker_at: Option<u64>,
) -> Unimock {
    let reads = reads.clone();
    let platform = SourceMock::platform
        .each_call(matching!())
        .returns(Platform::LinuxX64);
    let read = SourceMock::read
        .each_call(matching!())
        .answers_arc(Arc::new(move |_| {
            let second = clock.elapsed().as_secs();
            reads.lock().unwrap().push(second);
            if fail_at == Some(second) {
                return Err(ErrorKind::InvalidData.into());
            }
            Ok(Reading {
                at_ms: EPOCH_MS + 1_000 * second as i64,
                cpu: CpuReading::BusyMicros(500_000 * second),
                max_cpu_milli_vcpu: 1_000,
                memory_mib: 128 + second as u32,
                oom_kills: if second < 11 { 10 } else { 12 },
            })
        }));
    let Some(worker_at) = worker_at else {
        return Unimock::new((platform, read));
    };
    Unimock::new((
        platform,
        read,
        SourceMock::worker_present
            .each_call(matching!(101))
            .answers_arc(Arc::new(move |_, _| {
                Ok(clock.elapsed().as_secs() >= worker_at)
            })),
    ))
}

fn runner_at(clock: Instant, trace: &Trace, events: Vec<(u64, RunnerEvent)>) -> Unimock {
    let events = Arc::new(Mutex::new(VecDeque::from(events)));
    let trace = trace.clone();
    Unimock::new((
        RunnerMock::start
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                trace.lock().unwrap().push(Observation::Start);
                Ok(())
            })),
        RunnerMock::pid.each_call(matching!()).returns(Some(101)),
        RunnerMock::next_event
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let events = events.clone();
                Box::pin(async move {
                    let (offset_ms, _) = events.lock().unwrap().front().unwrap().clone();
                    sleep_until(clock + Duration::from_millis(offset_ms)).await;
                    Ok(events.lock().unwrap().pop_front().unwrap().1)
                })
            })),
    ))
}

fn usage(start_ms: i64, memory: Vec<u32>, kills: Vec<u32>) -> JobUsage {
    JobUsage {
        platform: Platform::LinuxX64,
        first_point_at_ms: EPOCH_MS + start_ms,
        point_seconds: 1,
        cpu_milli_vcpu: vec![500; memory.len()],
        peak_memory_mib: memory,
        oom_kills: Some(kills),
        steps: Vec::new(),
    }
}

fn kills(points: usize) -> Vec<u32> {
    let mut kills = vec![0; points];
    kills[5] = 2;
    kills
}

fn samples_with(last_memory: u32, points: usize, ended_at_ms: Option<i64>) -> Observation {
    let mut usage = usage(5_000, (134..=last_memory).collect(), kills(points));
    usage.steps = boat_steps(ended_at_ms);
    Observation::Message(GuestToAgent::UsageSamples {
        runner_id: RunnerId("runner-7".into()),
        usage: usage.into(),
    })
}

fn exit_with(code: i32, usage: JobUsage) -> Observation {
    Observation::Message(GuestToAgent::RunnerExited {
        runner_id: RunnerId("runner-7".into()),
        exit: RunnerExit { code: Some(code) },
        usage: Some(usage.into()),
    })
}
