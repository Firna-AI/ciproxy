//! In-memory boundary mocks and a lifecycle observation trace.

use std::collections::VecDeque;
use std::future::pending;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::HANDOFF_PROTOCOL_VERSION;
use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::identifiers::{ImageId, OrganizationId, RunnerId};
use ciproxy_protocol::runners::RunnerExit;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::Notify;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{ChannelMock, PowerMock, RunnerEvent, RunnerMock};
use crate::error::{Error, Result};
use crate::service::Supervisor;

#[derive(Debug, Eq, PartialEq)]
pub(in crate::service) enum Observation {
    Message(GuestToAgent),
    Start,
    Stop,
    Waiting,
    ControlLost,
    Poweroff,
}

pub(in crate::service) type Trace = Arc<Mutex<Vec<Observation>>>;

pub(in crate::service) fn configure() -> AgentToGuest {
    AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: RunnerId("runner-7".into()),
        organization_id: OrganizationId("org-1".into()),
        registration: Secret::new("jit-for-one-runner"),
        cache: None,
    }
}

pub(super) fn hello() -> Observation {
    Observation::Message(GuestToAgent::Hello {
        handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
        configuration_received: false,
        protocol_version: ProtocolVersion::CURRENT,
        image_id: ImageId("image-1".into()),
    })
}

pub(super) fn configured() -> Observation {
    Observation::Message(GuestToAgent::RunnerConfigured {
        runner_id: RunnerId("runner-7".into()),
    })
}

pub(super) fn exited(code: Option<i32>) -> Observation {
    Observation::Message(GuestToAgent::RunnerExited {
        runner_id: RunnerId("runner-7".into()),
        exit: RunnerExit { code },
    })
}

pub(super) fn channel(
    commands: Vec<Result<Option<AgentToGuest>>>,
    trace: &Trace,
    fail_exit: bool,
) -> Unimock {
    let commands = Arc::new(Mutex::new(VecDeque::from(commands)));
    let received = trace.clone();
    let sent = trace.clone();
    Unimock::new((
        ChannelMock::receive
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let commands = commands.clone();
                let received = received.clone();
                Box::pin(async move {
                    let command = commands.lock().unwrap().pop_front().expect("next command");
                    if !matches!(command, Ok(Some(_))) {
                        received.lock().unwrap().push(Observation::ControlLost);
                    }
                    command
                })
            })),
        ChannelMock::send
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, message| {
                let is_exit = matches!(message, GuestToAgent::RunnerExited { .. });
                sent.lock().unwrap().push(Observation::Message(message));
                if fail_exit && is_exit {
                    Err(Error::internal_message("callback unavailable"))
                } else {
                    Ok(())
                }
            })),
    ))
}

pub(super) fn runner(trace: &Trace, events: Vec<RunnerEvent>) -> Unimock {
    let started = trace.clone();
    let start = RunnerMock::start
        .some_call(matching!(_))
        .answers_arc(Arc::new(move |_, registration| {
            assert_eq!(registration, Secret::new("jit-for-one-runner"));
            started.lock().unwrap().push(Observation::Start);
            Ok(())
        }));
    let events = Arc::new(Mutex::new(VecDeque::from(events)));
    Unimock::new((
        start,
        RunnerMock::next_event
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let events = events.clone();
                Box::pin(async move { Ok(events.lock().unwrap().pop_front().expect("next event")) })
            })),
    ))
}

pub(super) fn shutdown_pair(trace: &Trace) -> (Unimock, Unimock) {
    let waiting = Arc::new(Notify::new());
    let command_gate = waiting.clone();
    let first = Arc::new(Mutex::new(Some(configure())));
    let sent = trace.clone();
    let channel = Unimock::new((
        ChannelMock::receive
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let configure = first.lock().unwrap().take();
                let command_gate = command_gate.clone();
                Box::pin(async move {
                    if let Some(configure) = configure {
                        return Ok(Some(configure));
                    }
                    command_gate.notified().await;
                    Ok(Some(AgentToGuest::Shutdown))
                })
            })),
        ChannelMock::send
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, message| {
                sent.lock().unwrap().push(Observation::Message(message));
                Ok(())
            })),
    ));
    let started = trace.clone();
    let stopped = trace.clone();
    let polled = trace.clone();
    let runner = Unimock::new((
        RunnerMock::start
            .some_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                started.lock().unwrap().push(Observation::Start);
                Ok(())
            })),
        RunnerMock::next_event
            .each_call(matching!())
            .answers_arc(Arc::new(move |_| {
                let waiting = waiting.clone();
                let polled = polled.clone();
                Box::pin(async move {
                    polled.lock().unwrap().push(Observation::Waiting);
                    waiting.notify_one();
                    pending().await
                })
            })),
        RunnerMock::stop
            .some_call(matching!())
            .answers_arc(Arc::new(move |_| {
                stopped.lock().unwrap().push(Observation::Stop);
                Ok(RunnerExit { code: None })
            })),
    ));
    (channel, runner)
}

pub(in crate::service) fn supervisor(
    channel: Unimock,
    runner: Unimock,
    trace: &Trace,
) -> Supervisor {
    let powered = trace.clone();
    Supervisor {
        image_id: ImageId("image-1".into()),
        channel: Box::new(channel),
        runner: Box::new(runner),
        power: Box::new(Unimock::new(
            PowerMock::shutdown
                .some_call(matching!())
                .answers_arc(Arc::new(move |_| {
                    powered.lock().unwrap().push(Observation::Poweroff);
                    Ok(())
                })),
        )),
    }
}
