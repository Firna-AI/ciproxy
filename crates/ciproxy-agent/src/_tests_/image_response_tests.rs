//! Existing command failure codes distinguish corrupt bytes from a bad bundle.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::ImagesMock;
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, spec, vm};
use crate::image_fixtures::image;
use crate::state::MachineState;

#[tokio::test]
async fn fetch_failures_use_the_existing_byte_verification_or_download_reason() {
    for (error, reason) in [
        (Error::ImageVerification, CommandFailure::ImageVerification),
        (Error::ImageLayout, CommandFailure::ImageDownload),
        (Error::HttpsRequired, CommandFailure::ImageDownload),
    ] {
        let state = MachineState {
            runners: vec![vm("job", true)],
            ..MachineState::default()
        };
        let agent = controller(
            state.clone(),
            Unimock::new(
                ImagesMock::fetch
                    .some_call(matching!(_))
                    .returns(Err(error)),
            ),
        );
        assert_eq!(
            agent
                .control(ControlMessage::FetchImage {
                    command_id: CommandId("image".to_owned()),
                    image: image()
                })
                .await
                .unwrap(),
            Some(AgentMessage::CommandResult {
                command_id: CommandId("image".to_owned()),
                outcome: CommandOutcome::Failed { reason },
            })
        );
        assert_eq!(*agent.state.lock().await, state);
    }
}

#[tokio::test]
async fn another_platform_fails_before_any_download_or_start() {
    let agent = controller(MachineState::default(), Unimock::new(()));
    let other = ImageDownload {
        platform: Platform::MacosArm64,
        ..image()
    };
    let mut runner = spec();
    runner.platform = Platform::MacosArm64;
    for message in [
        ControlMessage::FetchImage {
            command_id: CommandId("image".to_owned()),
            image: other,
        },
        ControlMessage::StartRunner {
            command_id: CommandId("image".to_owned()),
            runner,
        },
    ] {
        assert_eq!(
            agent.control(message).await.unwrap(),
            Some(AgentMessage::CommandResult {
                command_id: CommandId("image".to_owned()),
                outcome: CommandOutcome::Failed {
                    reason: CommandFailure::PlatformMismatch
                },
            })
        );
    }
    assert_eq!(*agent.state.lock().await, MachineState::default());
}
