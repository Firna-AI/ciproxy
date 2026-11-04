//! An absent guest cannot block repeated idempotent shutdown requests.

use std::path::PathBuf;
use std::sync::Arc;
use std::task::Poll;

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::secret::Secret;
use ciproxy_protocol::version::ProtocolVersion;
use futures_util::{pin_mut, poll};
use tokio::sync::{mpsc, watch};
use unimock::Unimock;

use crate::fixtures::vm;
use crate::layout::Layout;

use super::{Channel, Guests, LinuxGuests};

#[tokio::test]
async fn an_absent_guest_coalesces_shutdown_without_losing_configuration() {
    let mock = Arc::new(Unimock::new(()));
    let (events, _event_rx) = mpsc::channel(1);
    let guests = LinuxGuests::new(
        Layout {
            data: PathBuf::from("/fixture"),
            firecracker: PathBuf::from("/tools/firecracker"),
            jailer: PathBuf::from("/tools/jailer"),
        },
        mock.clone(),
        mock,
        events,
    );
    let record = vm("runner", false);
    let (sender, mut receiver) = mpsc::channel(16);
    let (shutdown, stop_requested) = watch::channel(false);
    let absent_guest = tokio::spawn(std::future::pending::<()>());
    guests.channels.lock().await.insert(
        record.local_id.clone(),
        Channel {
            sender,
            shutdown,
            task: absent_guest.abort_handle(),
        },
    );
    for request in 1..=64 {
        let send = guests.send(&record, &AgentToGuest::Shutdown);
        pin_mut!(send);
        assert!(
            matches!(poll!(send.as_mut()), Poll::Ready(Ok(()))),
            "shutdown request {request} must not wait for the guest"
        );
    }
    assert!(*stop_requested.borrow());
    let configuration = AgentToGuest::ConfigureRunner {
        protocol_version: ProtocolVersion::CURRENT,
        runner_id: record.report.runner_id.clone(),
        organization_id: record.report.organization_id.clone(),
        registration: Secret::new("registration"),
        cache: None,
    };
    guests.send(&record, &configuration).await.unwrap();
    assert_eq!(receiver.try_recv().unwrap(), configuration);
    assert_eq!(receiver.try_recv(), Err(mpsc::error::TryRecvError::Empty));
    guests.detach(&record).await.unwrap();
    let _ = absent_guest.await;
}
