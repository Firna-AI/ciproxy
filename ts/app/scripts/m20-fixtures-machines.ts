import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { machine } from "./fixtures-setup.ts";

type Machine = components["schemas"]["MachineView"];
type Slot = components["schemas"]["SlotView"];
type Job = components["schemas"]["JobView"];

export const machineIds = {
  primary: "30000000-0000-4000-8000-000000000001",
  offline: "30000000-0000-4000-8000-000000000002",
  noKvm: "30000000-0000-4000-8000-000000000003",
  noSlots: "30000000-0000-4000-8000-000000000004",
  diskLimited: "30000000-0000-4000-8000-000000000005",
  unknownCapacity: "30000000-0000-4000-8000-000000000006",
  draining: "30000000-0000-4000-8000-000000000007",
} as const;

/** The canonical dashboard jobs of the busy slots, apart from report IDs. */
export const slotJobIds = {
  test: "70000000-0000-4000-8000-000000000001",
  build: "70000000-0000-4000-8000-000000000002",
} as const;

const gigabyte = 1_000_000_000;
const bytes = (gb: number) => String(gb * gigabyte);

/** A running job of ciproxy that a runner took on forge-01. */
export function slotJob(overrides: Partial<Job> = {}): Job {
  return {
    actual_size: { vcpus: 2, memory_bytes: bytes(8), disk_bytes: bytes(81) },
    cache_counts: null,
    cache_source: null,
    completed_at: null,
    counts_as_start: true,
    git_ref: "refs/pull/1542/merge",
    github_url:
      "https://github.com/northstar-labs/control/actions/runs/9001/job/9101",
    head_branch: "cache-keys",
    id: slotJobIds.test,
    label: { name: "ciproxy", vcpus: 2, memory_gb: 8, disk_gb: 20 },
    location: {
      kind: "machine",
      id: machineIds.primary,
      name: "forge-01",
      first_slot: 0,
    },
    name: "test (rust)",
    organization: "northstar-labs",
    organization_id: "40000000-0000-4000-8000-000000000001",
    queued_at: "2026-11-06T11:55:47Z",
    repository: "northstar-labs/control",
    result: null,
    routing_reason: "A warm runner was ready, so the job started at once.",
    runtime_seconds: "252",
    started_at: "2026-11-06T11:55:48Z",
    status: "running",
    waited_seconds: "1",
    workflow: "CI",
    ...overrides,
  } satisfies Job;
}

/** A job of ciproxy-4vcpu that holds two slots, as one job. */
function buildJob(): Job {
  return slotJob({
    actual_size: { vcpus: 4, memory_bytes: bytes(16), disk_bytes: bytes(81) },
    git_ref: "refs/heads/main",
    head_branch: "main",
    id: slotJobIds.build,
    label: { name: "ciproxy-4vcpu", vcpus: 4, memory_gb: 16, disk_gb: 20 },
    location: {
      kind: "machine",
      id: machineIds.primary,
      name: "forge-01",
      first_slot: 2,
    },
    name: "build (linux-x64)",
    repository: "northstar-labs/runner",
    runtime_seconds: "210",
    started_at: "2026-11-06T11:56:30Z",
  });
}

function free(index: number): Slot {
  return {
    index: String(index),
    state: "free",
    runner_id: null,
    job_id: null,
    job: null,
  };
}

function warm(index: number, runner: string): Slot {
  return {
    index: String(index),
    state: "warm",
    runner_id: runner,
    job_id: null,
    job: null,
  };
}

/** Each report job ID differs from the dashboard job that the API matched. */
function occupancy(): Slot[] {
  const build = buildJob();
  return [
    {
      index: "0",
      state: "job",
      runner_id: "fixture-runner-1",
      job_id: "fixture-report-job-1",
      job: slotJob(),
    },
    warm(1, "fixture-runner-2"),
    ...[2, 3].map((index): Slot => ({
      index: String(index),
      state: "job",
      runner_id: "fixture-runner-4",
      job_id: "fixture-report-job-2",
      job: build,
    })),
    free(4),
    free(5),
  ];
}

export function fixtureMachines(): Machine[] {
  const primary = machine({
    id: machineIds.primary,
    hostname: "forge-01",
    processor: "AMD Ryzen 7 PRO 8700GE",
    threads: 16,
    memory_bytes: bytes(64),
    slots: 6,
    agent_version: "0.4.2",
    latest_agent_version: "0.4.2",
    agent_up_to_date: true,
    selected_image_disk_bytes: bytes(60),
    concurrent_default_jobs: "6",
    default_jobs_by_slots: "6",
    default_root_disk_bytes: bytes(81),
    job_disk_bytes: bytes(645),
    disk_limited: false,
    last_contact_at: fixtureTime,
    disks: [{ path: "/", capacity_bytes: bytes(1000), free_bytes: bytes(530) }],
    occupancy: occupancy(),
    warm_runners: 2,
    monthly_cost_cents: "5290",
    cache: {
      used_bytes: bytes(96),
      capacity_bytes: bytes(402),
      restores_last_7_days: "1000",
      hits_last_7_days: "840",
      docker_disk_bytes: bytes(18),
      docker_repositories: "3",
    },
  });
  return [
    primary,
    machine({
      ...primary,
      id: machineIds.offline,
      hostname: "forge-offline",
      offline: true,
      health_old: true,
      last_contact_at: "2026-11-06T09:12:00Z",
      agent_version: "0.0.9",
      agent_up_to_date: false,
      occupancy: [],
    }),
    machine({
      ...primary,
      id: machineIds.noKvm,
      hostname: "lab-no-kvm",
      kvm: false,
      slots: 0,
      concurrent_default_jobs: "0",
      default_jobs_by_slots: "0",
      occupancy: [],
      cache: null,
      monthly_cost_cents: null,
    }),
    machine({
      ...primary,
      id: machineIds.noSlots,
      hostname: "lab-no-slots",
      processor: "Intel Core i3-7100T",
      threads: 2,
      memory_bytes: bytes(8),
      slots: 0,
      concurrent_default_jobs: "0",
      default_jobs_by_slots: "0",
      occupancy: [],
      cache: null,
      monthly_cost_cents: null,
    }),
    machine({
      ...primary,
      id: machineIds.diskLimited,
      hostname: "office-mini-1",
      processor: "AMD Ryzen 7 7840HS",
      concurrent_default_jobs: "3",
      job_disk_bytes: bytes(279),
      disk_limited: true,
      disks: [
        { path: "/", capacity_bytes: bytes(512), free_bytes: bytes(362) },
      ],
      occupancy: [
        warm(0, "fixture-runner-5"),
        warm(1, "fixture-runner-6"),
        ...[2, 3, 4, 5].map(free),
      ],
      cache: {
        used_bytes: bytes(6),
        capacity_bytes: bytes(117),
        restores_last_7_days: "200",
        hits_last_7_days: "100",
        docker_disk_bytes: "0",
        docker_repositories: "0",
      },
      monthly_cost_cents: "0",
    }),
    machine({
      ...primary,
      id: machineIds.unknownCapacity,
      hostname: "forge-preparing",
      capacity_known: false,
      selected_image_disk_bytes: null,
      concurrent_default_jobs: null,
      default_root_disk_bytes: null,
      job_disk_bytes: null,
      disk_limited: null,
      latest_agent_version: null,
      agent_up_to_date: null,
      occupancy: [],
      cache: null,
    }),
    machine({
      ...primary,
      id: machineIds.draining,
      hostname: "forge-draining",
      draining: true,
      occupancy: [
        {
          index: "0",
          state: "job",
          runner_id: "fixture-runner-3",
          job_id: "fixture-report-job-3",
          job: null,
        },
        ...[1, 2, 3, 4, 5].map(free),
      ],
    }),
  ] satisfies Machine[];
}
