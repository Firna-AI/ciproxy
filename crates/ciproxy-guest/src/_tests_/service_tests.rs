//! Requirements of the one-runner guest lifecycle, without runtime I/O.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::GuestToAgent;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{RunnerEvent, RunnerMock};
use crate::error::Error;
use crate::service::Guest;

#[path = "support.rs"]
pub(super) mod support;

use support::{Observation, channel, configure, exited, hello, runner, shutdown_pair, supervisor};

#[tokio::test]
async fn reports_readiness_and_exit_before_poweroff() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let control = channel(vec![Ok(Some(configure())), Ok(None)], &trace, false);
    let process = runner(
        &trace,
        vec![
            RunnerEvent::Ready,
            RunnerEvent::Exited(RunnerExit { code: Some(1) }),
        ],
    );
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            Observation::Start,
            Observation::ControlLost,
            Observation::Message(GuestToAgent::RunnerReady {
                runner_id: RunnerId("runner-7".into()),
            }),
            exited(Some(1)),
            Observation::Poweroff,
        ]
    );
}

#[tokio::test]
async fn ignores_another_registration_in_the_same_guest() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let control = channel(
        vec![Ok(Some(configure())), Ok(Some(configure())), Ok(None)],
        &trace,
        false,
    );
    let process = runner(
        &trace,
        vec![RunnerEvent::Exited(RunnerExit { code: Some(0) })],
    );
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            Observation::Start,
            Observation::ControlLost,
            exited(Some(0)),
            Observation::Poweroff
        ]
    );
}

#[tokio::test]
async fn shutdown_stops_the_running_listener_and_reports_its_exit() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let (control, process) = shutdown_pair(&trace);
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            Observation::Start,
            Observation::Waiting,
            Observation::Stop,
            exited(None),
            Observation::Poweroff
        ]
    );
}

#[tokio::test]
async fn agent_connection_failure_keeps_the_runner_until_its_exit() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let control = channel(
        vec![
            Ok(Some(configure())),
            Err(Error::internal_message("agent restarted")),
        ],
        &trace,
        false,
    );
    let process = runner(
        &trace,
        vec![RunnerEvent::Exited(RunnerExit { code: Some(0) })],
    );
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            Observation::Start,
            Observation::ControlLost,
            exited(Some(0)),
            Observation::Poweroff
        ]
    );
}

#[tokio::test]
async fn failed_exit_report_still_powers_off_the_finished_guest() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let control = channel(vec![Ok(Some(configure())), Ok(None)], &trace, true);
    let process = runner(
        &trace,
        vec![RunnerEvent::Exited(RunnerExit { code: Some(0) })],
    );
    let mut guest = supervisor(control, process, &trace);

    guest.run().await.unwrap();

    assert_eq!(
        *trace.lock().unwrap(),
        vec![
            hello(),
            Observation::Start,
            Observation::ControlLost,
            exited(Some(0)),
            Observation::Poweroff
        ]
    );
}

#[tokio::test]
async fn failed_runner_start_reports_absent_exit_code_and_powers_off() {
    let trace = Arc::new(Mutex::new(Vec::new()));
    let control = channel(vec![Ok(Some(configure()))], &trace, false);
    let process = Unimock::new(
        RunnerMock::start
            .some_call(matching!(_))
            .returns(Err(Error::internal_message("executable unavailable")))
            .once(),
    );
    let mut guest = supervisor(control, process, &trace);

    let result = guest.run().await;

    assert!(matches!(result, Err(Error::Internal(_))));
    assert_eq!(
        *trace.lock().unwrap(),
        vec![hello(), exited(None), Observation::Poweroff]
    );
}
