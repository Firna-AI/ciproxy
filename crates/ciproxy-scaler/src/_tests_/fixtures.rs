//! Literal workspace facts and typed expected actions for scaler tests.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{
    JobId, LabelName, MachineId, OrganizationId, RunnerId, WorkspaceId,
};
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};

use crate::actions::{Action, Destination, StartReason, StartRunner};
use crate::provider::{
    AccountFacts, MonthlySpend, ProviderAccount, ProviderDispatch, ProviderId, ProviderRunner,
    ProviderSize, ProviderSizeId, Sandbox, SandboxId, StartBound,
};
use crate::registration::RegistrationTarget;
use crate::snapshot::{
    Label, Machine, NothingFits, Place, Runner, RunnerState, Snapshot, WaitingJob,
};
use ciproxy_protocol::platform::Platform;

pub(super) const BOAT: ProviderId = ProviderId::new("boat");
pub(super) const SECOND: ProviderId = ProviderId::new("second");

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
            platform: Platform::LinuxX64,
            id: MachineId("machine".into()),
            selected_image: None,
            connected: true,
            draining: false,
            capacity: Slots(6),
            disk_budget_bytes: 600 * GIGABYTE,
            docker_leased_bytes: Some(0),
            warm_count: 0,
            warm_allocation: Vec::new(),
            runners: Vec::new(),
        }],
        overflow_enabled: true,
        providers: vec![ProviderAccount {
            id: BOAT,
            available: true,
            refreshed: true,
            root_sizing: RootSizing::default(),
            facts: AccountFacts {
                connected: true,
                spend: MonthlySpend {
                    spent: 20,
                    limit: 100,
                    next_month_spent: Some(0),
                },
                sizes: vec![
                    boat_size("small", small(), 2),
                    boat_size("large", large(), 5),
                ],
                largest_size: Some(ProviderSizeId("large".into())),
                refusals: Vec::new(),
            },
        }],
        sandboxes: Vec::new(),
        overflow_wait: Duration::from_secs(30),
    }
}

pub(super) fn two_provider_snapshot() -> Snapshot {
    let mut state = snapshot();
    let mut second = state.providers[0].clone();
    second.id = SECOND;
    state.providers.push(second);
    for label in &mut state.labels {
        label.order.push(Place::Provider(SECOND));
    }
    state
}

pub(super) fn target(organization: &str, name: &str) -> RegistrationTarget {
    RegistrationTarget {
        organization_id: OrganizationId(organization.into()),
        label_id: name.into(),
        scale_set_id: format!("{organization}:{name}"),
        github_id: 53,
    }
}

pub(super) fn label(name: &str, size: VmSize) -> Label {
    Label {
        platform: Platform::LinuxX64,
        targets: vec![target("a", name), target("b", name)],
        name: LabelName(name.into()),
        size,
        order: vec![Place::OwnMachines, Place::Provider(BOAT)],
        nothing_fits: NothingFits::Wait,
    }
}

pub(super) fn queued(organization: &str, name: &str, age: u64) -> WaitingJob {
    WaitingJob {
        target: target(organization, name),
        queued_at: Duration::from_secs(100 - age),
    }
}

pub(super) fn runner(id: &str, organization: &str, name: &str, state: RunnerState) -> Runner {
    Runner {
        target: Some(target(organization, name)),
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
        provider: BOAT,
        id: SandboxId(id.into()),
        marked: true,
        runner: runner.map(|runner| ProviderRunner {
            required_root_disk_bytes: allocated(runner.requested_size).disk_bytes,
            runner,
            launched_at: None,
            dispatch: ProviderDispatch::Accepted,
        }),
    }
}

pub(super) fn boat_size(id: &str, size: VmSize, price: u64) -> ProviderSize {
    ProviderSize {
        id: ProviderSizeId(id.into()),
        size: allocated(size),
        price,
        start_bound: Some(StartBound::default()),
    }
}

pub(super) fn start(
    organization: &str,
    name: &str,
    size: VmSize,
    destination: Destination,
    reason: StartReason,
) -> Action {
    Action::StartRunner(Box::new(StartRunner {
        platform: Platform::LinuxX64,
        target: target(organization, name),
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        requested_size: size,
        size: allocated(size),
        destination,
        reason,
    }))
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
        provider_destination(BOAT, size_id),
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
    Action::StartRunner(Box::new(StartRunner {
        platform: Platform::LinuxX64,
        target: target(organization, name),
        organization_id: OrganizationId(organization.into()),
        label: LabelName(name.into()),
        requested_size,
        size: allocated(size),
        destination: provider_destination(BOAT, size_id),
        reason,
    }))
}

pub(super) fn provider_destination(provider: ProviderId, size_id: &str) -> Destination {
    Destination::Provider {
        provider,
        size_id: ProviderSizeId(size_id.into()),
    }
}

pub(super) fn allocated(size: VmSize) -> VmSize {
    RootSizing::default().vm_size(size).unwrap()
}
