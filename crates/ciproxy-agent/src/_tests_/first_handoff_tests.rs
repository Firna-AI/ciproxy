//! Reconnection cannot grant job-controlled input first-handoff authority.

use ciproxy_protocol::guest::JobDecision;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::mpsc;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::{CacheMock, StoreMock, VmsMock};
use crate::cache_authority_tests::{facts, original, starting};
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, spec, vm};
use crate::guest_transport::{GuestRead, GuestSocket, GuestSocketMock, GuestTransportMock};
use crate::layout::Layout;
use crate::process::{CommandsMock, UnitState};
use crate::state::MachineState;

use super::configuration_delivery_tests::{hello, socket};
use super::{Guests, LinuxGuests};

#[tokio::test(start_paused = true)]
async fn reconnect_before_first_handoff_permanently_uses_github_even_after_configuration_retry() {
    let record = vm("job", false);
    let first = Box::new(Unimock::new((
        GuestSocketMock::receive.stub(|each| {
            each.call(matching!())
                .answers(&|_| Box::pin(async { Ok(hello(false)) }) as GuestRead<'_>)
                .once()
                .then()
                .answers(&|_| Box::pin(async { Err(Error::Disconnected) }) as GuestRead<'_>);
        }),
        GuestSocketMock::send
            .some_call(matching!(_))
            .returns(Ok(())),
    ))) as Box<dyn GuestSocket>;
    let mut runtime = original();
    runtime.results_url = ciproxy_protocol::secret::Secret::new("http://169.254.169.254/");
    runtime.permissions.write = true;
    let second = socket(
        vec![hello(false), starting(runtime.clone()), starting(runtime)],
        Some(Arc::new(Mutex::new(Vec::new()))),
    );
    let sockets = Arc::new(Mutex::new(VecDeque::from([first, second])));
    let transport = Unimock::new(
        GuestTransportMock::connect
            .each_call(matching!(_))
            .answers_arc(Arc::new(move |_, _| {
                Ok(sockets.lock().unwrap().pop_front().unwrap())
            })),
    );
    let (events, mut observed) = mpsc::channel(16);
    let mut guests = LinuxGuests::new(
        Layout {
            data: PathBuf::from("/fixture"),
            firecracker: PathBuf::new(),
            jailer: PathBuf::new(),
        },
        Arc::new(Unimock::new(
            CommandsMock::unit
                .each_call(matching!(_))
                .answers(&|_, _| Ok(UnitState::Active)),
        )),
        events,
    );
    guests.transport = Arc::new(transport);
    guests
        .attach(
            &record,
            Some(AgentToGuest::ConfigureRunner {
                protocol_version: ProtocolVersion::CURRENT,
                runner_id: record.report.runner_id.clone(),
                organization_id: record.report.organization_id.clone(),
                registration: spec().registration,
                cache: None,
            }),
        )
        .await
        .unwrap();
    let handoffs = Arc::new(Mutex::new(Vec::new()));
    let handed = handoffs.clone();
    let observer = Arc::new(
        Unimock::new((
            CacheMock::handoff.stub(|each| {
                each.call(matching!(_, _, _))
                    .answers_arc(Arc::new(move |_, _, _, runtime| {
                        handed.lock().unwrap().push(runtime);
                        Ok(())
                    }))
                    .at_least_times(0);
            }),
            CacheMock::prepare
                .some_call(matching!(_, _))
                .returns(Ok(None)),
        ))
        .no_verify_in_drop(),
    );
    let mut agent = controller(MachineState { runners: vec![record.clone()], ..MachineState::default() }, Unimock::new((
        StoreMock::save.each_call(matching!(_)).answers(&|_, _| Ok(())),
        VmsMock::send.each_call(matching!((_, message) if matches!(message, AgentToGuest::PrepareJob { decision: JobDecision::GitHubOnly, .. }))).answers(&|_, _, _| Ok(())),
    )));
    agent.cache = observer;
    for _ in 0..4 {
        let event = observed.recv().await.unwrap();
        agent
            .guest(event.provenance, "job", event.message)
            .await
            .unwrap();
    }
    agent.control(facts()).await.unwrap();
    assert_eq!(handoffs.lock().unwrap().len(), 0);
    guests.detach(&record).await.unwrap();
}
