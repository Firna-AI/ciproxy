//! Gateway identities, issued secrets, and machine management values.

use chrono::{DateTime, Utc};
use ciproxy_protocol::agent::ControlMessage;
use ciproxy_protocol::secret::Secret;
use ciproxy_store_interface::gateway::MachineSnapshot;
use uuid::Uuid;

use crate::connections::Outbox;

/// Authority and connection identity established by authentication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Session {
    /// Workspace authorized by the credential.
    pub workspace_id: Uuid,
    /// Machine authorized by the credential.
    pub machine_id: Uuid,
    /// Fencing identifier replaced on each connection.
    pub id: Uuid,
}

/// A completed authentication and replies sent before ordinary commands.
pub struct Handshake {
    /// Authenticated connection scope.
    pub session: Session,
    /// Registration or welcome, followed by desired drain reconciliation.
    pub replies: Vec<ControlMessage>,
    /// Mailbox attached atomically with this local authenticated session.
    pub outbox: Box<dyn Outbox>,
}

/// An install token whose plaintext is returned only to its caller.
#[derive(Debug)]
pub struct IssuedToken {
    /// Record identifier used to revoke this token.
    pub id: Uuid,
    /// Secret that adds machines within its exclusive lifetime.
    pub token: Secret,
    /// Exclusive expiry one hour after server issuance.
    pub expires_at: DateTime<Utc>,
}

/// An Admin action on one machine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MachineAction {
    /// Finish jobs, stop warm runners, and refuse new jobs.
    Drain,
    /// Allow new jobs and warm runners again.
    Resume,
    /// Revoke credentials and finish jobs before the agent stops.
    Remove,
    /// Clear machine cache entries without deleting Docker disks.
    ClearCache,
}

/// The result of command routing to an active local connection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Delivery {
    /// The active connection accepted the command for delivery.
    Connected,
    /// There is no local connection. Durable drain or removal still applies.
    Offline,
}

/// Contact freshness calculated with the gateway's clock.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    /// The last contact was less than sixty seconds ago.
    Online,
    /// There was no contact for sixty seconds or no report exists.
    Offline,
}

/// An active registration with its last report and current freshness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MachineView {
    /// Persisted latest health and desired machine settings.
    pub snapshot: MachineSnapshot,
    /// Offline reports remain available and have old health values.
    pub presence: Presence,
}
