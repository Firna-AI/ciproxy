import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";
import { machine } from "./fixtures-setup.ts";

type Machine = components["schemas"]["MachineView"];
type Slot = components["schemas"]["SlotView"];

export const machineIds = {
  primary: "30000000-0000-4000-8000-000000000001",
  offline: "30000000-0000-4000-8000-000000000002",
  noKvm: "30000000-0000-4000-8000-000000000003",
  noSlots: "30000000-0000-4000-8000-000000000004",
  diskLimited: "30000000-0000-4000-8000-000000000005",
  unknownCapacity: "30000000-0000-4000-8000-000000000006",
  draining: "30000000-0000-4000-8000-000000000007",
} as const;

function occupancy(): Slot[] {
  return [
    {
      index: "0",
      state: "job",
      runner_id: "fixture-runner-1",
      job_id: "fixture-job-1",
    },
    { index: "1", state: "warm", runner_id: "fixture-runner-2", job_id: null },
    ...["2", "3", "4", "5"].map((index): Slot => ({
      index,
      state: "free",
      runner_id: null,
      job_id: null,
    })),
  ];
}

export function fixtureMachines(): Machine[] {
  const primary = machine({
    id: machineIds.primary,
    hostname: "forge-01",
    processor: "AMD Ryzen 7 PRO 8700GE",
    threads: 16,
    memory_bytes: "64000000000",
    slots: 6,
    concurrent_default_jobs: "6",
    last_contact_at: fixtureTime,
    disks: [
      {
        path: "/",
        capacity_bytes: "1000000000000",
        free_bytes: "530000000000",
      },
    ],
    occupancy: occupancy(),
    monthly_cost_cents: "5290",
    cache: {
      used_bytes: "96000000000",
      capacity_bytes: "420000000000",
      restores_last_7_days: "1000",
      hits_last_7_days: "840",
      docker_disk_bytes: "18000000000",
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
      occupancy: [],
      warm_runners: 0,
    }),
    machine({
      ...primary,
      id: machineIds.noKvm,
      hostname: "lab-no-kvm",
      kvm: false,
      slots: 0,
      concurrent_default_jobs: "0",
      occupancy: [],
      warm_runners: 0,
      cache: null,
      monthly_cost_cents: null,
    }),
    machine({
      id: machineIds.noSlots,
      hostname: "lab-no-slots",
      threads: 2,
      memory_bytes: "8000000000",
      slots: 0,
      concurrent_default_jobs: "0",
      occupancy: [],
      warm_runners: 0,
    }),
    machine({
      ...primary,
      id: machineIds.diskLimited,
      hostname: "office-mini-1",
      concurrent_default_jobs: "1",
      disks: [
        {
          path: "/",
          capacity_bytes: "250000000000",
          free_bytes: "80000000000",
        },
      ],
      occupancy: [],
      warm_runners: 0,
      cache: {
        used_bytes: "6000000000",
        capacity_bytes: "82500000000",
        restores_last_7_days: "200",
        hits_last_7_days: "100",
        docker_disk_bytes: "0",
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
      occupancy: [],
      warm_runners: 0,
      cache: null,
    }),
    machine({
      ...primary,
      id: machineIds.draining,
      hostname: "forge-draining",
      draining: true,
      warm_runners: 0,
      occupancy: [
        {
          index: "0",
          state: "job",
          runner_id: "fixture-runner-3",
          job_id: "fixture-job-2",
        },
      ],
    }),
  ] satisfies Machine[];
}
