//! Existing command failure codes distinguish corrupt bytes from a bad bundle.

use ciproxy_protocol::agent::{AgentMessage, CommandFailure, CommandOutcome, ControlMessage};
use ciproxy_protocol::identifiers::CommandId;
use unimock::{MockFn, Unimock, matching};

use crate::boundaries::ImagesMock;
use crate::controller::Agent;
use crate::error::Error;
use crate::fixtures::{controller, vm};
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
