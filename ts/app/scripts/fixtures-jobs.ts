import type { components } from "../../api/index.ts";
import { fixtureTime } from "./fixtures-core.ts";

type JobView = components["schemas"]["JobView"];

export const fixtureJobIds = {
  machine: "50000000-0000-4000-8000-000000000101",
  boat: "50000000-0000-4000-8000-000000000102",
  unknown: "50000000-0000-4000-8000-000000000103",
  failed: "50000000-0000-4000-8000-000000000104",
  cancelled: "50000000-0000-4000-8000-000000000105",
  untimed: "50000000-0000-4000-8000-000000000106",
  older: "50000000-0000-4000-8000-000000000107",
  running: "50000000-0000-4000-8000-000000000201",
  waiting: "50000000-0000-4000-8000-000000000202",
} as const;

export const fixtureMachineId = "30000000-0000-4000-8000-000000000001";
export const fixtureLabelId = "60000000-0000-4000-8000-000000000001";
export const fixtureVmSize = {
  vcpus: 4,
  memory_bytes: "8000000000",
  disk_bytes: "60000000000",
} satisfies components["schemas"]["JobVmSize"];

export function fixtureJob(overrides: Partial<JobView> = {}): JobView {
  return {
    id: fixtureJobIds.machine,
    name: "Build and test",
    workflow: "CI",
    repository: "northstar-labs/control",
    organization: "northstar-labs",
    organization_id: "40000000-0000-4000-8000-000000000001",
    label: { name: "ciproxy", vcpus: 4, memory_gb: 8, disk_gb: 20 },
    location: {
      kind: "machine",
      id: fixtureMachineId,
      name: "forge-01",
      first_slot: 0,
    },
    actual_size: fixtureVmSize,
    status: "completed",
    result: "success",
    counts_as_start: true,
    queued_at: "2026-11-06T11:50:00Z",
    started_at: "2026-11-06T11:50:00Z",
    completed_at: "2026-11-06T11:56:12Z",
    waited_seconds: "0",
    runtime_seconds: "372",
    git_ref: "refs/heads/main",
    head_branch: "main",
    github_url:
      "https://github.com/northstar-labs/control/actions/runs/701/job/801",
    routing_reason: "A warm runner was ready on forge-01.",
    cache_source: "machine",
    cache_counts: { restored_bytes: "2100000000", restored_entries: "3" },
    ...overrides,
  } satisfies JobView;
}

export function historyJobs(): JobView[] {
  return [
    fixtureJob(),
    fixtureJob({
      id: fixtureJobIds.boat,
      name: "Browser checks",
      workflow: "E2E",
      location: { kind: "boat", size: "4vcpu-8gb" },
      cache_source: "github",
      cache_counts: null,
      waited_seconds: "31.5",
      queued_at: "2026-11-06T11:49:28.500Z",
      routing_reason: "Own machines were full. The job started on Boat.",
    }),
    fixtureJob({
      id: fixtureJobIds.unknown,
      name: "Legacy request",
      label: {
        name: "ciproxy-old",
        vcpus: null,
        memory_gb: null,
        disk_gb: null,
      },
      git_ref: null,
      head_branch: null,
      cache_counts: null,
      cache_source: null,
    }),
    fixtureJob({
      id: fixtureJobIds.failed,
      name: "Lint",
      repository: "orbit-labs/worker",
      organization: "orbit-labs",
      organization_id: "40000000-0000-4000-8000-000000000002",
      result: "failed",
      cache_source: "machine_and_github",
      cache_counts: { restored_bytes: "0", restored_entries: "0" },
    }),
    fixtureJob({
      id: fixtureJobIds.cancelled,
      name: "Cancelled before a runner",
      result: "cancelled",
      counts_as_start: false,
      location: null,
      actual_size: null,
      started_at: null,
      runtime_seconds: null,
      waited_seconds: null,
      cache_source: null,
      cache_counts: null,
      routing_reason: null,
    }),
    fixtureJob({
      id: fixtureJobIds.untimed,
      name: "Completed with unknown timing",
      result: "succeeded",
      completed_at: null,
      started_at: null,
      runtime_seconds: null,
      waited_seconds: null,
      github_url: null,
    }),
    fixtureJob({
      id: fixtureJobIds.older,
      name: "Older retained job",
      queued_at: "2026-10-10T11:50:00Z",
      started_at: "2026-10-10T11:50:00Z",
      completed_at: "2026-10-10T11:56:12Z",
    }),
  ];
}

export function visibleHistory(historyDays: number): JobView[] {
  const since = new Date(
    Date.parse(fixtureTime) - historyDays * 86_400_000,
  ).toISOString();
  return historyJobs().filter((job) => {
    const at = job.completed_at ?? job.started_at ?? job.queued_at;
    return (
      at !== undefined && at !== null && Date.parse(at) >= Date.parse(since)
    );
  });
}

export function filteredHistory(
  historyDays: number,
  query: URLSearchParams,
): JobView[] {
  const search = (query.get("search") ?? "").toLowerCase();
  const place = query.get("place");
  const from = query.get("from");
  const until = query.get("until");
  return visibleHistory(historyDays).filter((job) => {
    const day = (job.completed_at ?? job.started_at ?? job.queued_at)?.slice(
      0,
      10,
    );
    return (
      [job.name, job.workflow, job.repository, job.organization].some((value) =>
        value.toLowerCase().includes(search),
      ) &&
      (place === null || job.location?.kind === place) &&
      (from === null || (day !== undefined && day >= from)) &&
      (until === null || (day !== undefined && day < until))
    );
  });
}
