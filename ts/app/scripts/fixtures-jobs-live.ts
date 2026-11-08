import type { components } from "../../api/index.ts";
import {
  fixtureTemporaryRetryAt,
  fullQueueJobs,
} from "./fixtures-jobs-polling.ts";
import {
  fixtureJob,
  fixtureJobIds,
  fixtureLabelId,
  fixtureVmSize,
} from "./fixtures-jobs.ts";

export const jobsModes = [
  "live",
  "quiet",
  "unknown-waiting",
  "nothing-fits",
  "unknown-fit",
  "unknown-request",
  "spend-limit",
  "boat-refused",
  "temporary-retry",
  "overflow-unknown",
] as const;
export type JobsMode = (typeof jobsModes)[number];

export function liveJobs(
  mode: JobsMode,
  role: components["schemas"]["WorkspaceRole"],
  banner: components["schemas"]["PageBanner"] | null,
): components["schemas"]["JobsLivePage"] {
  const canManage = role === "admin";
  const quiet = mode === "quiet";
  const unknown = mode === "unknown-waiting";
  const noFit = mode === "nothing-fits";
  const unknownFit = mode === "unknown-fit";
  const temporaryRetry = mode === "temporary-retry";
  const label = {
    name: noFit ? "ciproxy-large" : "ciproxy",
    vcpus: noFit ? 128 : 4,
    memory_gb: noFit ? 256 : 8,
    disk_gb: noFit ? 1000 : 20,
  } satisfies components["schemas"]["JobLabel"];
  const waiting =
    quiet || unknown
      ? []
      : [
          fixtureJob({
            id: fixtureJobIds.waiting,
            name: "Waiting build",
            label,
            status: "queued",
            counts_as_start: false,
            queued_at: "2026-11-06T11:59:30Z",
            started_at: null,
            completed_at: null,
            result: null,
            location: null,
            actual_size: null,
            waited_seconds: "30",
            runtime_seconds: null,
            cache_source: null,
            cache_counts: null,
            routing_reason: null,
          }),
        ];
  return {
    banner,
    can_manage: canManage,
    on_boat: "0",
    overflow_enabled: true,
    overflow_wait_seconds: 30,
    own_slots_total: unknownFit ? null : "6",
    own_slots_used: unknownFit
      ? null
      : quiet
        ? "0"
        : temporaryRetry
          ? "6"
          : "1",
    warm_runners: quiet ? "2" : "0",
    running: quiet
      ? []
      : temporaryRetry
        ? fullQueueJobs()
        : [
            fixtureJob({
              id: fixtureJobIds.running,
              status: "running",
              result: null,
              started_at: "2026-11-06T11:59:10Z",
              queued_at: "2026-11-06T11:59:10Z",
              completed_at: null,
              runtime_seconds: "50",
              ...(mode === "unknown-request"
                ? {
                    label: {
                      name: "ciproxy-old",
                      vcpus: null,
                      memory_gb: null,
                      disk_gb: null,
                    },
                  }
                : {}),
            }),
          ],
    waiting,
    waiting_count: unknown ? null : String(waiting.length),
    visibility: unknown
      ? [
          {
            organization: "northstar-labs",
            suspended: true,
            removed: false,
            github_url:
              "https://github.com/organizations/northstar-labs/settings/installations/9001",
          },
        ]
      : [],
    starting_runners:
      quiet ||
      unknown ||
      noFit ||
      unknownFit ||
      mode === "spend-limit" ||
      mode === "boat-refused" ||
      temporaryRetry ||
      mode === "overflow-unknown"
        ? []
        : [
            {
              id: "80000000-0000-4000-8000-000000000001",
              label_id: fixtureLabelId,
              boat: true,
              machine_id: null,
            },
          ],
    waiting_labels:
      waiting.length === 0
        ? []
        : [
            {
              label_id: fixtureLabelId,
              label_name: label.name,
              waiting_jobs: String(waiting.length),
              can_edit: canManage,
              ask_admin: !canManage && noFit,
              nothing_fits: unknownFit ? null : noFit,
              current: {
                id: fixtureLabelId,
                label: {
                  name: label.name,
                  vcpus: noFit ? 128 : 4,
                  memory_gb: noFit ? 256 : 8,
                  disk_gb: noFit ? 1000 : 20,
                  allow_boat: true,
                  allow_machines: true,
                  when_nothing_fits: "wait",
                },
                fit: {
                  eligible_machines: 1,
                  fitting_machines: noFit || unknownFit ? 0 : 1,
                  no_machine_fits: noFit,
                  unavailable_machines: unknownFit ? 1 : 0,
                  minimum_jobs: noFit || unknownFit ? null : "6",
                  maximum_jobs: noFit || unknownFit ? null : "6",
                },
              },
              boat: unknownFit
                ? { state: "unavailable" }
                : noFit
                  ? { state: "no_size_fits" }
                  : {
                      state: "fitting",
                      size: fixtureVmSize,
                      size_name: "4vcpu-8gb",
                      refused: mode === "boat-refused",
                    },
            },
          ],
    overflow:
      mode === "overflow-unknown"
        ? null
        : {
            billing_required: false,
            connected: true,
            counted_cents: mode === "spend-limit" ? "10000" : "1200",
            limit_cents: "10000",
            limit_reached: mode === "spend-limit",
            key_rejected: false,
            measured_complete: true,
            next_month: "2026-12-01T00:00:00Z",
            refused_sizes: mode === "boat-refused" ? ["4vcpu-8gb"] : [],
            retry_at:
              mode === "boat-refused"
                ? "2026-11-06T13:00:00Z"
                : temporaryRetry
                  ? fixtureTemporaryRetryAt
                  : null,
          },
  } satisfies components["schemas"]["JobsLivePage"];
}
