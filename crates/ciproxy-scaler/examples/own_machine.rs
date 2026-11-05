//! A no-secret smoke test that starts one runner on free own capacity.

use std::time::Duration;

use ciproxy_agent::root_sizing::RootSizing;
use ciproxy_protocol::identifiers::{LabelName, MachineId, OrganizationId, WorkspaceId};
use ciproxy_protocol::resources::{GIGABYTE, Slots, VmSize};
use ciproxy_scaler::actions::{Action, Destination, StartReason, StartRunner};
use ciproxy_scaler::boat::{BoatAccount, MonthlySpend};
use ciproxy_scaler::decision::decide;
use ciproxy_scaler::snapshot::{Destinations, Label, Machine, NothingFits, Snapshot, WaitingJob};

fn main() {
    let size = VmSize {
        vcpu: 2,
        memory_bytes: 8 * GIGABYTE,
        disk_bytes: 20 * GIGABYTE,
    };
    let snapshot = Snapshot {
        root_sizing: RootSizing::default(),
        workspace_id: WorkspaceId("example-workspace".into()),
        now: Duration::from_secs(100),
        control_plane_available: true,
        organizations: vec![OrganizationId("example-org".into())],
        labels: vec![Label {
            name: LabelName("ciproxy".into()),
            size,
            destinations: Destinations {
                own_machines: true,
                boat: true,
            },
            nothing_fits: NothingFits::Wait,
        }],
        queue: vec![WaitingJob {
            organization_id: OrganizationId("example-org".into()),
            label: LabelName("ciproxy".into()),
            queued_at: Duration::from_secs(100),
        }],
        machines: vec![Machine {
            id: MachineId("example-machine".into()),
            connected: true,
            draining: false,
            capacity: Slots(1),
            disk_budget_bytes: 82_500_000_000,
            warm_count: 0,
            warm_allocation: Vec::new(),
            runners: Vec::new(),
        }],
        boat: BoatAccount {
            enabled: false,
            connected: false,
            spend: MonthlySpend {
                spent: 0,
                limit: 100,
            },
            sizes: Vec::new(),
            largest_size: None,
            refusals: Vec::new(),
        },
        sandboxes: Vec::new(),
        overflow_wait: Duration::from_secs(30),
    };
    let actions = decide(&snapshot);
    assert_eq!(
        actions,
        vec![Action::StartRunner(StartRunner {
            organization_id: OrganizationId("example-org".into()),
            label: LabelName("ciproxy".into()),
            requested_size: size,
            size: VmSize {
                disk_bytes: 81_635_856_384,
                ..size
            },
            destination: Destination::Machine(MachineId("example-machine".into())),
            reason: StartReason::OwnMachineCapacity,
        })]
    );
    println!("{actions:#?}");
}
