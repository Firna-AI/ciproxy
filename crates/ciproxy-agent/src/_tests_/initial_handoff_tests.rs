//! Boot connection attempts do not consume the one configuration-channel grant.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::version::ProtocolVersion;
use tokio::sync::mpsc;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::GuestProvenance;
use crate::cache_authority_tests::{original, starting};
use crate::error::Error;
use crate::fixtures::{spec, vm};
use crate::guest_transport::GuestTransportMock;
use crate::layout::Layout;
use crate::process::{CommandsMock, UnitState};

use super::configuration_delivery_tests::{hello, socket};
use super::{Guests, LinuxGuests};

#[tokio::test(start_paused = true)]
async fn boot_connection_failure_keeps_first_authority_until_configuration_is_sent() {
    let record = vm("job", false);
    let sent = Arc::new(Mutex::new(Vec::new()));
    let peer = socket(vec![hello(false), starting(original())], Some(sent.clone()));
    let peer = Arc::new(Mutex::new(Some(peer)));
    let transport = Unimock::new(GuestTransportMock::connect.stub(|each| {
        each.call(matching!(_))
            .answers(&|_, _| Err(Error::Disconnected))
            .once()
            .then()
            .answers_arc(Arc::new(move |_, _| {
                Ok(peer.lock().unwrap().take().unwrap())
            }))
            .once();
    }));
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
    let configure = AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: record.report.runner_id.clone(),
        organization_id: record.report.organization_id.clone(),
        registration: spec().registration,
        cache: None,
    };
    guests
        .attach(&record, Some(configure.clone()))
        .await
        .unwrap();
    observed.recv().await.unwrap();
    let handoff = observed.recv().await.unwrap();
    assert_eq!(handoff.provenance, GuestProvenance::InitialConfiguration);
    assert_eq!(*sent.lock().unwrap(), vec![configure]);
    guests.detach(&record).await.unwrap();
}
