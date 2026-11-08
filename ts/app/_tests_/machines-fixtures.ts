// Test data for the Machines tests. Every body uses the generated contract.

import type { OverflowSettings } from "../src/api/machine-schema.ts";
import type { MachineView, SlotView } from "../src/api/schema.ts";

export const nbsp = "\u00a0";

/** A job slot, a warm slot, or a free slot with its own index. */
export function slot(
  index: number,
  state: SlotView["state"],
  runner: string | null = state === "free" ? null : `runner-${index}`,
  job: string | null = state === "job" ? `job-${index}` : null,
): SlotView {
  return {
    index: String(index),
    state,
    runner_id: runner,
    job_id: job,
    job: null,
  };
}

export function cache(
  hits: string,
  restores: string,
  overrides: Partial<NonNullable<MachineView["cache"]>> = {},
): NonNullable<MachineView["cache"]> {
  return {
    capacity_bytes: "191000000000",
    docker_disk_bytes: "96000000000",
    docker_repositories: "3",
    hits_last_7_days: hits,
    restores_last_7_days: restores,
    used_bytes: "83000000000",
    ...overrides,
  };
}

export function summary(
  overrides: Partial<OverflowSettings> = {},
): OverflowSettings {
  return {
    account: {
      connected_at: "2026-09-15T09:00:00Z",
      name: "mokly",
      rejected_at: null,
    },
    available: true,
    enabled: true,
    next_month_at: "2026-12-01T00:00:00Z",
    spend: {
      bounded: true,
      confirmed_cents: "6120",
      limit_reached: false,
      settled_bound_cents: "0",
      unconfirmed_cents: "0",
    },
    spend_limit_cents: "20000",
    wait_seconds: 30,
    ...overrides,
  };
}
