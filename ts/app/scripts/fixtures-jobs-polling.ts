import type { components } from "../../api/index.ts";
import { fixtureJob, fixtureMachineId } from "./fixtures-jobs.ts";

export const fixtureTemporaryRetryAt = "2026-11-06T12:01:00Z";

export function fullQueueJobs(): components["schemas"]["JobView"][] {
  return Array.from({ length: 6 }, (_, slot) =>
    fixtureJob({
      id: `50000000-0000-4000-8000-00000000021${slot}`,
      name: `Running build ${slot + 1}`,
      status: "running",
      result: null,
      started_at: "2026-11-06T11:59:10Z",
      queued_at: "2026-11-06T11:59:10Z",
      completed_at: null,
      runtime_seconds: "50",
      location: {
        kind: "machine",
        id: fixtureMachineId,
        name: "forge-01",
        first_slot: slot,
      },
    }),
  );
}
