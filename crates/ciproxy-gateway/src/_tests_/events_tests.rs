//! Lifecycle callbacks can deliver commands through the same service.

use std::sync::{Arc, Mutex, Weak, mpsc};
use std::thread;
use std::time::Duration;

use ciproxy_protocol::agent::{AgentMessage, ControlMessage};
use ciproxy_protocol::identifiers::{CommandId, OrganizationId, RunnerId};
use ciproxy_store_interface::gateway::GatewayStoreMock;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::{AgentEventsMock, ClockMock};
use crate::connections::ConnectionsMock;
use crate::gateway::MachineGateway;
use crate::types::{Delivery, Session};

use super::support::{at, gateway, id};

#[test]
fn allows_lifecycle_callback_to_route_a_command_without_deadlock() {
    let reference: Arc<Mutex<Option<Weak<dyn MachineGateway>>>> = Arc::new(Mutex::new(None));
    let captured = reference.clone();
    let session = Session {
        workspace_id: id(1),
        machine_id: id(3),
        id: id(4),
    };
    let service = gateway(
        Unimock::new((
            GatewayStoreMock::machine_session
                .each_call(matching!(_, _))
                .answers(&|_, _, _| Ok(Some(id(4)))),
            GatewayStoreMock::record_activity
                .some_call(matching!((workspace, machine, session, time)
                if *workspace == id(1) && *machine == id(3) && *session == id(4) && *time == at()))
                .returns(Ok(())),
        )),
        Unimock::new(()),
        Unimock::new(ClockMock::now.some_call(matching!()).returns(at())),
        Unimock::new(()),
        Unimock::new(
            ConnectionsMock::send
                .some_call(matching!((actual, message)
            if actual.id == id(4) && matches!(message, ControlMessage::StopRunner { .. })))
                .returns(Ok(Delivery::Connected)),
        ),
        Unimock::new(
            AgentEventsMock::receive
                .some_call(matching!(_, AgentMessage::RunnerReady { .. }))
                .answers_arc(Arc::new(move |_, scope: Session, _| {
                    let service = captured
                        .lock()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .upgrade()
                        .unwrap();
                    let delivered = service.send(
                        scope.workspace_id,
                        scope.machine_id,
                        ControlMessage::StopRunner {
                            command_id: CommandId("retire-ready".into()),
                            organization_id: OrganizationId("org-1".into()),
                            runner_id: RunnerId("runner-1".into()),
                        },
                    )?;
                    assert_eq!(delivered, Delivery::Connected);
                    Ok(())
                })),
        ),
    );
    *reference.lock().unwrap() = Some(Arc::downgrade(&service));
    let (sender, receiver) = mpsc::channel();
    let worker_service = service.clone();
    let worker = thread::spawn(move || {
        let result = worker_service.receive(
            session,
            AgentMessage::RunnerReady {
                runner_id: RunnerId("runner-1".into()),
            },
        );
        sender.send(result).unwrap();
    });
    let result = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("lifecycle callback hung");
    assert_eq!(result.unwrap(), ());
    worker.join().unwrap();
}
