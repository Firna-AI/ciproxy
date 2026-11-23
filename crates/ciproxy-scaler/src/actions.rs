//! Typed intents for the control plane to apply against a fresh snapshot.

use std::time::Duration;

use ciproxy_protocol::identifiers::{LabelName, MachineId, OrganizationId, RunnerId};
use ciproxy_protocol::platform::Platform;
use ciproxy_protocol::resources::VmSize;

use crate::provider::{ProviderId, ProviderSizeId, SandboxId};
use crate::registration::RegistrationTarget;

/// One requested change, with no external work performed by the scaler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Action {
    /// Register and start one new ephemeral runner, boxed to keep actions small.
    StartRunner(Box<StartRunner>),
    /// Retire an idle own-machine runner after confirming it is still idle.
    StopRunner {
        /// Runner whose registration must be removed before releasing its VM.
        runner_id: RunnerId,
        /// Why the idle runner is no longer needed.
        reason: StopReason,
    },
    /// Delete a marked sandbox after confirming it has no running job.
    DeleteSandbox {
        /// Provider that owns this sandbox.
        provider: ProviderId,
        /// Sandbox that ciproxy created.
        sandbox_id: SandboxId,
        /// Why the sandbox is no longer needed.
        reason: StopReason,
    },
}

/// A runner belongs to an organization and label, never a job or repository.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartRunner {
    /// Saved exact binding selected for this start.
    pub target: RegistrationTarget,
    /// Organization whose scale set will register the runner.
    pub organization_id: OrganizationId,
    /// Label of the runner's scale set.
    pub label: LabelName,
    /// Platform of the label, which the destination machine must have.
    pub platform: Platform,
    /// Label dimensions at registration, retained separately from provider size.
    pub requested_size: VmSize,
    /// Actual VM dimensions, including the image and ext4 root allowance.
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
    /// An available size in one provider account.
    Provider {
        /// Provider that will start the runner.
        provider: ProviderId,
        /// Size to start in that provider's catalog.
        size_id: ProviderSizeId,
    },
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
    /// The provider comes before a fitting own machine in the label order.
    ProviderBeforeOwnMachines,
    /// No place fits and the label allows the largest provider size.
    LargestProviderSize,
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
    /// The actual provider VM size no longer meets the label's current size policy.
    SizeNoLongerAllowed,
    /// A launched provider runner took no job within its ready deadline.
    ReadyDeadlinePassed,
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
