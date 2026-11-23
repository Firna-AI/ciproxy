// The Macs of the `macs` scenario of the Machines and Settings harness, and
// the macOS labels that run on them. A Mac uses the slot rule of a Linux
// machine and runs at most 2 job VMs at once. mac-studio-1 has 6 slots and
// runs a job and a warm runner, its 2 VMs. mac-mini-1 has room for 1 slot.
// mac-old has a macOS older than the job image, so it cannot start VMs. A
// macOS label runs on the Macs only, so its Boat fit is disabled.

import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { machine } from "./fixtures-setup.ts";

type Schemas = components["schemas"];
type Slot = Schemas["SlotView"];

const gb = (value: number) => String(value * 1_000_000_000);

export const macIds = {
  studio: "30000000-0000-4000-8000-000000000101",
  mini: "30000000-0000-4000-8000-000000000102",
  old: "30000000-0000-4000-8000-000000000103",
} as const;

function slots(states: readonly Slot["state"][]): Slot[] {
  return states.map((state, index) => ({
    index: String(index),
    state,
    runner_id: state === "free" ? null : `mac-runner-${index}`,
    job_id: null,
    job: null,
  }));
}

/** What every Mac of the scenario reports, apart from its size and slots. */
const macBase = {
  platform: "macos_arm64",
  vm_limit: 2,
  agent_version: "0.4.2",
  latest_agent_version: "0.4.2",
  agent_up_to_date: true,
  last_contact_at: fixtureTime,
  selected_image_disk_bytes: gb(120),
  default_root_disk_bytes: gb(141),
  disk_limited: false,
  warm_runners: 1,
  monthly_cost_cents: null,
  cache: {
    // A Mac has no machine cache yet, so the API reports it unavailable.
    machine_cache: { status: "unavailable" },
    docker_disks: { status: "measured", used_bytes: "0", repositories: "0" },
  },
} as const satisfies Partial<Schemas["MachineView"]>;

export function macMachines(): Schemas["MachineView"][] {
  const mini = machine({
    ...macBase,
    id: macIds.mini,
    hostname: "mac-mini-1",
    processor: "Apple M4",
    threads: 10,
    memory_bytes: gb(16),
    slots: 1,
    concurrent_default_jobs: "1",
    default_jobs_by_slots: "1",
    job_disk_bytes: gb(279),
    disks: [{ path: "/", capacity_bytes: gb(512), free_bytes: gb(232) }],
    occupancy: slots(["job"]),
  });
  return [
    machine({
      ...macBase,
      id: macIds.studio,
      hostname: "mac-studio-1",
      processor: "Apple M4 Max",
      threads: 16,
      memory_bytes: gb(64),
      slots: 6,
      concurrent_default_jobs: "2",
      default_jobs_by_slots: "6",
      job_disk_bytes: gb(645),
      disks: [{ path: "/", capacity_bytes: gb(1000), free_bytes: gb(590) }],
      occupancy: slots(["job", "warm", "free", "free", "free", "free"]),
    }),
    mini,
    {
      ...mini,
      id: macIds.old,
      hostname: "mac-old",
      kvm: false,
      virtualization: "operating_system_too_old",
      slots: 0,
      concurrent_default_jobs: "0",
      default_jobs_by_slots: "0",
      occupancy: [],
    },
  ];
}

/** The fit of a macOS size on the Macs: 2 jobs on mac-studio-1, 1 on mini. */
export function macFit(): Schemas["LabelFit"] {
  const offer = { slots: "6", vcpus: "12", memory_bytes: gb(48) };
  return {
    boat: { state: "disabled" },
    boat_connected: true,
    boat_required_root_disk_bytes: null,
    eligible_machines: 2,
    fitting_machines: 2,
    largest_boat: null,
    machines: [
      {
        ...offer,
        id: macIds.studio,
        hostname: "mac-studio-1",
        job_disk_bytes: gb(645),
        required_root_disk_bytes: gb(141),
        concurrent_jobs: "2",
      },
      {
        ...offer,
        slots: "1",
        id: macIds.mini,
        hostname: "mac-mini-1",
        job_disk_bytes: gb(279),
        required_root_disk_bytes: gb(141),
        concurrent_jobs: "1",
      },
    ],
    minimum_jobs: "1",
    maximum_jobs: "2",
    no_machine_fits: false,
    placement: "machines_only",
    unavailable_machines: 0,
  };
}

/** The default macOS labels, with the fit of the Macs. */
export function macLabels(): Schemas["LabelView"][] {
  const base = {
    platform: "macos_arm64",
    allow_machines: true,
    allow_boat: false,
    when_nothing_fits: "wait",
    disk_gb: 20,
  } as const;
  return [
    {
      id: "60000000-0000-4000-8000-000000000101",
      label: { ...base, name: "ciproxy-macos", vcpus: 2, memory_gb: 8 },
      fit: macFit(),
      jobs_last_30_days: "1204",
      job_count_since: "2026-10-07T12:00:00Z",
      job_count_until: fixtureTime,
    },
    {
      id: "60000000-0000-4000-8000-000000000102",
      label: { ...base, name: "ciproxy-macos-medium", vcpus: 4, memory_gb: 16 },
      fit: {
        ...macFit(),
        fitting_machines: 1,
        minimum_jobs: "2",
        maximum_jobs: "2",
      },
      jobs_last_30_days: "318",
      job_count_since: "2026-10-07T12:00:00Z",
      job_count_until: fixtureTime,
    },
  ];
}
