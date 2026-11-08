//! Unconfigured shutdown uses the reset path supported by x86 Firecracker.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::ImageId;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::ChannelMock;
use crate::power::{LinuxPower, ShutdownCommandMock};
use crate::service::{Guest, Supervisor};

#[tokio::test]
async fn shutdown_of_unconfigured_guest_requests_vm_reset_without_a_runner() {
    let commands = Arc::new(Mutex::new(Vec::new()));
    let observed = commands.clone();
    let mut guest = Supervisor {
        image_id: ImageId("image-1".into()),
        channel: Box::new(Unimock::new((
            ChannelMock::send.some_call(matching!(_)).returns(Ok(())),
            ChannelMock::receive
                .some_call(matching!())
                .answers(&|_| Box::pin(async { Ok(Some(AgentToGuest::Shutdown)) })),
        ))),
        runner: Box::new(Unimock::new(())),
        power: Box::new(LinuxPower {
            command: Box::new(Unimock::new(
                ShutdownCommandMock::run
                    .some_call(matching!(_))
                    .answers_arc(Arc::new(move |_, program| {
                        observed.lock().unwrap().push(program.to_owned());
                        Ok(true)
                    })),
            )),
        }),
    };

    guest.run().await.unwrap();

    assert_eq!(*commands.lock().unwrap(), vec!["/sbin/reboot"]);
}
