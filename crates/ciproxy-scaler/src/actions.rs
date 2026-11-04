//! Typed intents for the control plane to apply against a fresh snapshot.

use std::time::Duration;

use ciproxy_protocol::identifiers::{LabelName, MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::resources::VmSize;

use crate::boat::{BoatSizeId, SandboxId};

/// One requested change, with no external work performed by the scaler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Register and start one new ephemeral runner.
    StartRunner(StartRunner),
    /// Retire an idle own-machine runner after confirming it is still idle.
    StopRunner {
        /// Runner whose registration must be removed before releasing its VM.
        runner_id: RunnerId,
        /// Why the idle runner is no longer needed.
        reason: StopReason,
    },
    /// Delete a marked sandbox after confirming it has no running job.
    DeleteSandbox {
        /// Sandbox that ciproxy created.
        sandbox_id: SandboxId,
        /// Why the sandbox is no longer needed.
        reason: StopReason,
    },
}

/// A runner belongs to an organization and label, never a job or repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRunner {
    /// Organization whose scale set will register the runner.
    pub organization_id: OrganizationId,
    /// Label of the runner's scale set.
    pub label: LabelName,
    /// Label dimensions at registration, retained separately from Boat size.
    pub requested_size: VmSize,
    /// Actual VM size, which can differ from the label on Boat.
    pub size: VmSize,
    /// Machine or catalog size to use.
    pub destination: Destination,
    /// The routing rule that requested this runner.
    pub reason: StartReason,
}

/// Where the caller must start a runner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Destination {
    /// A connected own machine with enough free capacity.
    Machine(MachineId),
    /// An available size in the customer's Boat account.
    Boat(BoatSizeId),
}

/// The observable cause of a runner start.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StartReason {
    /// A waiting job can use free capacity on an own machine.
    OwnMachineCapacity,
    /// A machine has a vacancy in its shared warm allocation.
    WarmPool,
    /// Eligible own machines are busy and the overflow wait has elapsed.
    OverflowWaitElapsed {
        /// Age of the waiting demand at this tick.
        waited: Duration,
    },
    /// No connected, non-draining, allowed own machine fits the size.
    NoEligibleMachine,
    /// Neither destination fits and the label allows a smaller Boat VM.
    LargestBoatSize,
}

/// Why an idle runner or sandbox must retire.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopReason {
    /// The organization is no longer active in this workspace.
    OrganizationRemoved,
    /// The label was removed or its size changed.
    LabelChanged,
    /// The label no longer allows this destination.
    DestinationDisabled,
    /// The own machine is draining.
    MachineDraining,
    /// A waiting job needs the slots or disk held by a warm runner.
    WarmPreemption,
    /// The runner exceeds or differs from the machine's warm allocation.
    WarmAllocationChanged,
    /// No waiting demand needs this runner.
    NoWaitingJob,
    /// The runner exited, including when no job was assigned.
    RunnerExited,
    /// A marked sandbox has no runner and no running job.
    OrphanSandbox,
}
