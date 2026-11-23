import { overflowSettings } from "../scripts/fixtures-boat.ts";
import type { ProviderSettings } from "../src/api/schema.ts";
// Test data for the Machines tests. Every body uses the generated contract.

import type { OverflowSettings } from "../src/api/machine-schema.ts";
import type { CacheView, SlotView } from "../src/api/schema.ts";
import type {
  MeasuredCache,
  MeasuredDocker,
} from "../src/view/machines-cache.ts";

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

/** A machine cache that the machine measured: 83 of 191 GB in use. */
export function measuredCache(hits: string, restores: string): MeasuredCache {
  return {
    capacity_bytes: "191000000000",
    hits_last_7_days: hits,
    restores_last_7_days: restores,
    status: "measured",
    used_bytes: "83000000000",
  };
}

/** Docker disks that the machine measured. */
export function docker(
  repositories: string,
  usedBytes = "96000000000",
): MeasuredDocker {
  return { repositories, status: "measured", used_bytes: usedBytes };
}

/** A part of a stored report that the machine did not measure. */
export const unavailable = { status: "unavailable" } as const;

/** A stored report: a measured machine cache, and 3 Docker disks of 96 GB. */
export function cache(
  hits: string,
  restores: string,
  overrides: Partial<CacheView> = {},
): CacheView {
  return {
    docker_disks: docker("3"),
    machine_cache: measuredCache(hits, restores),
    ...overrides,
  };
}

export function summary(
  overrides: Partial<OverflowSettings> & Partial<ProviderSettings> = {},
): OverflowSettings {
  return overflowSettings({
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
  });
}
