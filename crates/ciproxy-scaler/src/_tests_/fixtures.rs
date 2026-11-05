//! Literal workspace facts and typed expected actions for scaler tests.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{
    JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::{Action, Destination, StartReason, StartRunner};
use crate::boat::{BoatAccount, BoatSize, BoatSizeId, MonthlySpend, Sandbox, SandboxId};
use crate::snapshot::{
    Destinations, Label, Machine, NothingFits, Runner, RunnerState, Snapshot, WaitingJob,
};

pub(super) fn small() -> VmSize {
    VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 20 * GIGABYTE,
    }
}

pub(super) fn large() -> VmSize {
    VmSize {
        vcpu: 4,
        memory_bytes: 16 * GIGABYTE,
        disk_bytes: 20 * GIGABYTE,
    }
}

pub(super) fn snapshot() -> Snapshot {
    Snapshot {
        workspace_id: WorkspaceId("workspace".into()),
        root_sizing: RootSizing::default(),
        now: Duration::from_secs(100),
        control_plane_available: true,
        organizations: vec![OrganizationId("a".into()), OrganizationId("b".into())],
        labels: vec![label("ciproxy", small()), label("large", large())],
        queue: Vec::new(),
        machines: vec![Machine {
            id: MachineId("machine".into()),
            connected: true,
            draining: false,
            capacity: Slots(6),
            disk_budget_bytes: 600 * GIGABYTE,
            warm_count: 0,
            warm_allocation: Vec::new(),
            runners: Vec::new(),
        }],
        boat: BoatAccount {
            enabled: true,
            connected: true,
            spend: MonthlySpend {
                spent: 20,
                limit: 100,
            },
            sizes: vec![
                boat_size("small", small(), 2),
                boat_size("large", large(), 5),
            ],
            largest_size: Some(BoatSizeId("large".into())),
            refusals: Vec::new(),
        },
        sandboxes: Vec::new(),
        overflow_wait: Duration::from_secs(30),
    }
}

pub(super) fn label(name: &str, size: VmSize) -> Label {
    Label {
        name: LabelName(name.into()),
        size,
        destinations: Destinations {
            own_machines: true,
            boat: true,
        },
        nothing_fits: NothingFits::Wait,
    }
}

pub(super) fn queued(organization: &str, name: &str, age: u64) -> WaitingJob {
    WaitingJob {
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        queued_at: Duration::from_secs(100 - age),
    }
}

pub(super) fn runner(id: &str, organization: &str, name: &str, state: RunnerState) -> Runner {
    Runner {
        id: RunnerId(id.into()),
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        requested_size: small(),
        size: allocated(small()),
        state,
    }
}

pub(super) fn running(id: &str) -> Runner {
    runner(
        id,
        "a",
        "ciproxy",
        RunnerState::Running(JobId("job".into())),
    )
}

pub(super) fn warm(id: &str, organization: &str) -> Runner {
    runner(id, organization, "ciproxy", RunnerState::Ready)
}

pub(super) fn sandbox(id: &str, runner: Option<Runner>) -> Sandbox {
    Sandbox {
        id: SandboxId(id.into()),
        marked: true,
        runner,
    }
}

pub(super) fn boat_size(id: &str, size: VmSize, price: u64) -> BoatSize {
    BoatSize {
        id: BoatSizeId(id.into()),
        size: allocated(size),
        price,
    }
}

pub(super) fn start(
    organization: &str,
    name: &str,
    size: VmSize,
    destination: Destination,
    reason: StartReason,
) -> Action {
    Action::StartRunner(StartRunner {
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        requested_size: size,
        size: allocated(size),
        destination,
        reason,
    })
}

pub(super) fn own(organization: &str, name: &str, size: VmSize) -> Action {
    start(
        organization,
        name,
        size,
        Destination::Machine(MachineId("machine".into())),
        StartReason::OwnMachineCapacity,
    )
}

pub(super) fn boat(
    organization: &str,
    name: &str,
    size: VmSize,
    size_id: &str,
    reason: StartReason,
) -> Action {
    start(
        organization,
        name,
        size,
        Destination::Boat(BoatSizeId(size_id.into())),
        reason,
    )
}

pub(super) fn boat_with_request(
    organization: &str,
    name: &str,
    requested_size: VmSize,
    size: VmSize,
    size_id: &str,
    reason: StartReason,
) -> Action {
    Action::StartRunner(StartRunner {
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        requested_size,
        size: allocated(size),
        destination: Destination::Boat(BoatSizeId(size_id.into())),
        reason,
    })
}

pub(super) fn allocated(size: VmSize) -> VmSize {
    RootSizing::default().vm_size(size).unwrap()
}
