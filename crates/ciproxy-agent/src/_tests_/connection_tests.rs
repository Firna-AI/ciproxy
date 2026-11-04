//! Only the gateway's exact credential-rejection close can remove a machine.

use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

use crate::connection::decode_frame;
use crate::error::Error;

#[derive(Debug, Eq, PartialEq)]
enum Outcome {
    Revoked,
    Disconnected,
    Unexpected,
}

#[test]
fn only_the_exact_gateway_policy_rejection_revokes_the_credential() {
    for (scenario, close, expected) in [
        (
            "permanent credential rejection",
            Some((CloseCode::Policy, "[gateway] authentication failed")),
            Outcome::Revoked,
        ),
        (
            "unsupported protocol version",
            Some((CloseCode::Policy, "[gateway] unsupported protocol version")),
            Outcome::Disconnected,
        ),
        (
            "connection replaced",
            Some((CloseCode::Policy, "[gateway] connection replaced")),
            Outcome::Disconnected,
        ),
        (
            "same reason with an ordinary close code",
            Some((CloseCode::Normal, "[gateway] authentication failed")),
            Outcome::Disconnected,
        ),
        (
            "different reason with the same prefix",
            Some((
                CloseCode::Policy,
                "[gateway] authentication failed: details",
            )),
            Outcome::Disconnected,
        ),
        ("no close payload", None, Outcome::Disconnected),
    ] {
        let frame = close.map(|(code, reason)| CloseFrame {
            code,
            reason: reason.into(),
        });
        let outcome = match decode_frame(Message::Close(frame)) {
            Err(Error::Revoked) => Outcome::Revoked,
            Err(Error::Disconnected) => Outcome::Disconnected,
            _ => Outcome::Unexpected,
        };
        assert_eq!(outcome, expected, "{scenario}");
    }
}
