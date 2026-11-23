//! Worker log observations do not control supervision or usage sampling.

use std::io::{Cursor, ErrorKind};
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::{JobId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::runtime::{CachePermissions, RuntimeCredentials};
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::usage::{JobStep, JobStepTiming};
use tokio::time::Instant;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::RunnerEvent;
use crate::service::Guest;
use crate::usage::Usage;
use crate::usage::worker_log::LogPagesMock;

use super::{Observation, Trace, channel, configure, configured, exit_with, hello, runner_at};
use super::{source, supervisor, usage};

const PAGE: &str = "Worker_20231114-221325-utc.log";
pub(super) const BOAT_LOG: &str = concat!(
    "[2023-11-14 22:13:26Z INFO StepsRunner] Processing step: DisplayName='Boat step'\n",
    "[2023-11-14 22:13:26Z INFO StepsRunner] Starting the step.\n",
    "[2023-11-14 22:14:00Z INFO StepsRunner] Step result: Succeeded\n",
);
const VALID: &str = concat!(
    "[2023-11-14 22:13:31Z INFO StepsRunner] Processing step: DisplayName='Own step'\n",
    "[2023-11-14 22:13:31Z INFO StepsRunner] Starting the step.\n",
    "[2023-11-14 22:13:32Z INFO StepsRunner] Step result: Succeeded\n",
);
const INVALID: &str =
    "[2023-11-14 22:13:33Z INFO StepsRunner] Processing step: DisplayName='Unclosed name\n";

#[tokio::test(start_paused = true)]
async fn missing_or_broken_worker_log_keeps_the_same_samples_and_exit() {
    let broken = format!("{VALID}{INVALID}").into_bytes();
    for (case, contents, expected_steps) in [
        ("missing", None, vec![]),
        ("broken", Some(broken), vec![]),
        (
            "valid",
            Some(VALID.as_bytes().to_vec()),
            vec![ran("Own step", 1_700_000_011_000, Some(1_700_000_012_000))],
        ),
    ] {
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
        let process = runner_at(
            clock,
            &trace,
            vec![
                (
                    10_000,
                    RunnerEvent::Starting {
                        job_id: job_id.clone(),
                        runtime: runtime.clone(),
                    },
                ),
                (13_500, RunnerEvent::Exited(RunnerExit { code: Some(7) })),
            ],
        );
        let control = channel(vec![Ok(Some(configure())), Ok(None)], &trace, false);
        let mut guest = supervisor(control, process, &trace);
        let pages = Unimock::new((
            LogPagesMock::list
                .each_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    Ok(if clock.elapsed().is_zero() {
                        Vec::new()
                    } else {
                        vec![PAGE.into()]
                    })
                })),
            LogPagesMock::open
                .each_call(matching!((name, 0) if *name == PAGE))
                .answers_arc(Arc::new(move |_, _, _| {
                    let bytes = contents.clone().ok_or(ErrorKind::NotFound)?;
                    Ok(Box::new(Cursor::new(bytes)))
                })),
        ));
        guest.usage = Some(Usage::new(
            Box::new(source(clock, &reads, None, None)),
            None,
            Box::new(pages),
        ));

        guest.run().await.unwrap();

        let mut final_usage = usage(10_000, vec![139, 140, 141], vec![2, 0, 0]);
        final_usage.steps = expected_steps;
        assert_eq!(*reads.lock().unwrap(), [10, 11, 12, 13], "{case}");
        assert_eq!(
            *trace.lock().unwrap(),
            [
                hello(),
                configured(),
                Observation::Start,
                Observation::ControlLost,
                Observation::Message(GuestToAgent::JobStarting {
                    runner_id: RunnerId("runner-7".into()),
                    job_id,
                    runtime,
                }),
                exit_with(7, final_usage),
                Observation::Poweroff,
            ],
            "{case}"
        );
    }
}

pub(super) fn worker_pages(clock: Instant, contents: fn(u64) -> Option<&'static str>) -> Unimock {
    Unimock::new((
        LogPagesMock::list
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let second = clock.elapsed().as_secs();
                Ok(if second > 0 && contents(second).is_some() {
                    vec![PAGE.into()]
                } else {
                    Vec::new()
                })
            })),
        LogPagesMock::open
            .each_call(matching!((name, _) if *name == PAGE))
            .answers_arc(Arc::new(move |_, _, offset| {
                let text = contents(clock.elapsed().as_secs()).ok_or(ErrorKind::NotFound)?;
                let mut reader = Cursor::new(text);
                reader.set_position(offset);
                Ok(Box::new(reader))
            })),
    ))
}

pub(super) fn boat_steps(end: Option<i64>) -> Vec<JobStep> {
    vec![ran("Boat step", 1_700_000_006_000, end)]
}

fn ran(name: &str, start: i64, end: Option<i64>) -> JobStep {
    JobStep {
        name: name.into(),
        timing: JobStepTiming::Ran {
            started_at_ms: Some(start),
            ended_at_ms: end,
        },
    }
}
