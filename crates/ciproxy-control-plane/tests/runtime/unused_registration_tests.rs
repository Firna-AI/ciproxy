//! GitHub keeps an unused JIT registration as an offline runner, so ciproxy removes it.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use ciproxy_github::registrations::{RegistrationRetirement, RunnerRegistrationsMock};
use ciproxy_protocol::agent::AgentMessage;
use ciproxy_protocol::identifiers::RunnerId;
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_store_interface::runners::RunnerState;
use unimock::{MockFn, Unimock, matching};

use super::clock::clock;
use super::recovery_support::{runtime_with_clock, saved};
use super::settled::Settled;
use super::support::Fixture;

fn exited(fixture: &Fixture) -> AgentMessage {
    AgentMessage::RunnerExited {
        runner_id: RunnerId(fixture.intent.runner.id.to_string()),
        exit: RunnerExit { code: Some(1) },
        usage: None,
    }
}

#[tokio::test]
async fn a_warm_runner_that_exits_without_a_job_loses_its_registration() {
    let fixture = Fixture::new();
    let name = format!("ciproxy-{}", fixture.intent.runner.id);
    let removals = Arc::new(AtomicUsize::new(0));
    let counted = removals.clone();
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::retire
            .each_call(matching!(_, _, Some(47)))
            .answers_arc(Arc::new(move |_, _, runner: &str, _| {
                assert_eq!(runner, name);
                counted.fetch_add(1, Ordering::SeqCst);
                Ok(RegistrationRetirement::Removed { id: 47 })
            })),
    ));
    let runtime = runtime_with_clock(
        &fixture,
        registrations,
        Arc::new(Unimock::new(())),
        clock(false),
    );
    runtime
        .settled_agent(fixture.workspace, fixture.machine, exited(&fixture))
        .await
        .unwrap();
    assert_eq!(removals.load(Ordering::SeqCst), 1);
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
}

#[tokio::test]
async fn a_busy_answer_keeps_a_stopped_runner_stopped() {
    let fixture = Fixture::new();
    let registrations = Arc::new(Unimock::new(
        RunnerRegistrationsMock::retire
            .each_call(matching!(_, _, Some(47)))
            .answers(&|_, _, _, _| Ok(RegistrationRetirement::Busy { id: 47 })),
    ));
    let runtime = runtime_with_clock(
        &fixture,
        registrations,
        Arc::new(Unimock::new(())),
        clock(false),
    );
    runtime
        .settled_agent(fixture.workspace, fixture.machine, exited(&fixture))
        .await
        .unwrap();
    assert_eq!(saved(&fixture).runner.state, RunnerState::Stopped);
}
