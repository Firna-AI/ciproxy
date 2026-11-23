//! Unconfigured shutdown uses the reset path supported by x86 Firecracker,
//! and a power-off on macOS, where a reboot would start the VM again.

use std::sync::{Arc, Mutex};

use ciproxy_protocol::guest::AgentToGuest;
use ciproxy_protocol::identifiers::ImageId;
use unimock::{MockFn, Unimock, matching};

use crate::boundary::ChannelMock;
use crate::platform::Platform;
use crate::power::{ShutdownCommandMock, VmPower};
use crate::service::{Guest, Supervisor};

#[tokio::test]
async fn shutdown_of_unconfigured_guest_requests_vm_reset_without_a_runner() {
    assert_eq!(end_commands(Platform::Linux).await, vec!["/sbin/reboot"]);
}

#[tokio::test]
async fn shutdown_of_unconfigured_mac_guest_powers_off_without_a_runner() {
    assert_eq!(
        end_commands(Platform::MacOs).await,
        vec!["/sbin/shutdown -h now"]
    );
}

async fn end_commands(platform: Platform) -> Vec<String> {
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
        usage: None,
        power: Box::new(VmPower {
            platform,
            command: Box::new(Unimock::new(
                ShutdownCommandMock::run
                    .some_call(matching!(_, _))
                    .answers_arc(Arc::new(move |_, program, arguments| {
                        let mut command = vec![program.to_owned()];
                        command.extend(arguments.iter().map(|argument| (*argument).to_owned()));
                        observed.lock().unwrap().push(command.join(" "));
                        Ok(true)
                    })),
            )),
        }),
    };

    guest.run().await.unwrap();

    commands.lock().unwrap().clone()
}
