//! Guest receipt decides recovery without a startup timer or persisted JIT.

use ciproxy_protocol::guest::HANDOFF_PROTOCOL_VERSION;
use std::future::pending;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::{AgentToGuest, GuestToAgent};
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::mpsc;
use unimock::{MockFn, Unimock, matching};

use crate::error::Error;
use crate::fixtures::{spec, vm};
use crate::guest_transport::{GuestRead, GuestSocket, GuestSocketMock, GuestTransportMock};
use crate::layout::Layout;
use crate::process::{CommandsMock, UnitState};

use super::{Guests, LinuxGuests};

pub(super) fn hello(received: bool) -> GuestToAgent {
    GuestToAgent::Hello {
        handoff_protocol_version: HANDOFF_PROTOCOL_VERSION,
        protocol_version: ProtocolVersion::CURRENT,
        image_id: vm("runner-1", false).report.image_id,
        configuration_received: received,
    }
}

pub(super) fn socket(
    messages: Vec<GuestToAgent>,
    sent: Option<Arc<Mutex<Vec<AgentToGuest>>>>,
) -> Box<dyn GuestSocket> {
    let messages = Arc::new(Mutex::new(std::collections::VecDeque::from(messages)));
    let receive = GuestSocketMock::receive
        .each_call(matching!())
        .answers_arc(Arc::new(move |_| {
            let messages = messages.clone();
            let future: GuestRead<'_> = Box::pin(async move {
                let result = messages.lock().unwrap().pop_front();
                match result {
                    Some(message) => Ok(message),
                    None => pending().await,
                }
            });
            future
        }));
    match sent {
        Some(sent) => Box::new(Unimock::new((
            receive,
            GuestSocketMock::send
                .each_call(matching!(_))
                .answers_arc(Arc::new(move |_, message| {
                    sent.lock().unwrap().push(message.clone());
                    Ok(())
                })),
        ))),
        None => Box::new(Unimock::new(receive)),
    }
}

#[tokio::test(start_paused = true)]
async fn dropped_configuration_write_replays_until_the_guest_confirms_receipt() {
    let record = vm("runner-1", false);
    let sent = Arc::new(Mutex::new(Vec::new()));
    let first_sent = sent.clone();
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
            .answers_arc(Arc::new(move |_, message| {
                first_sent.lock().unwrap().push(message.clone());
                Ok(())
            })),
    ))) as Box<dyn GuestSocket>;
    let second = socket(
        vec![
            hello(false),
            GuestToAgent::RunnerConfigured {
                runner_id: record.report.runner_id.clone(),
            },
        ],
        Some(sent.clone()),
    );
    let sockets = Arc::new(Mutex::new(std::collections::VecDeque::from([
        first, second,
    ])));
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
    let request = spec();
    let configuration = AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: record.report.runner_id.clone(),
        organization_id: record.report.organization_id.clone(),
        registration: request.registration,
        cache: None,
    };
    guests
        .attach(&record, Some(configuration.clone()))
        .await
        .unwrap();
    let first = observed.recv().await.unwrap();
    assert!(!guests.configuration_lost(&record).await.unwrap());
    let second = observed.recv().await.unwrap();
    let receipt = observed.recv().await.unwrap();
    assert_eq!(
        (
            first.configuration_lost,
            second.configuration_lost,
            receipt.configuration_lost
        ),
        (false, false, false)
    );
    assert_eq!(
        *sent.lock().unwrap(),
        vec![configuration.clone(), configuration]
    );
    assert!(
        guests.channels.lock().await[&record.local_id]
            .configuration
            .lock()
            .await
            .pending
            .is_none()
    );
    guests.detach(&record).await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn restored_guest_hello_distinguishes_lost_configuration_from_configured_recovery() {
    for (received, expected) in [(false, vec![true, true]), (true, vec![false])] {
        let record = vm("runner-1", false);
        let transport = Unimock::new(
            GuestTransportMock::connect
                .some_call(matching!(_))
                .returns(Ok(socket(vec![hello(received)], None))),
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
        guests.attach(&record, None).await.unwrap();
        let mut replayed = Vec::new();
        for _ in 0..expected.len() {
            replayed.push(observed.recv().await.unwrap().configuration_lost);
        }
        assert_eq!(replayed, expected);
        assert_eq!(guests.configuration_lost(&record).await.unwrap(), !received);
        guests.detach(&record).await.unwrap();
    }
}
