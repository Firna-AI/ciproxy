// The usage of a job for the tests of the CPU and memory panel: figures,
// steps with times after the first sample, and a measured job of 5 seconds
// in points of 2 seconds, so its last point is short.

import type { JobUsageStep } from "../src/api/jobs-types.ts";
import type { MeasuredUsage } from "../src/view/jobs-usage.ts";

/** The words of a value. Formats keep a value on one line with no-break spaces. */
export function plain(value: string | null): string | null {
  return value === null ? null : value.replaceAll("\u00a0", " ");
}

export function figures(cpu: number, memory: string, kills: number | null = 0) {
  return {
    average_cpu_milli_vcpu: cpu,
    peak_memory_bytes: memory,
    oom_kills: kills,
  };
}

/** A UTC time, in seconds after the first sample of the chart. */
export function at(seconds: number): string {
  return new Date(
    Date.parse("2026-11-06T12:00:00Z") + seconds * 1000,
  ).toISOString();
}

export function step(
  number: number,
  name: string,
  from: number | null,
  to: number | null,
  overrides: Partial<JobUsageStep> = {},
): JobUsageStep {
  return {
    number,
    name,
    script: false,
    skipped: false,
    started_at: from === null ? null : at(from),
    ended_at: to === null ? null : at(to),
    duration_seconds: from === null || to === null ? null : String(to - from),
    figures: null,
    ...overrides,
  };
}

/** A job of 5 seconds in points of 2 seconds, so the last point is short. */
export function measured(
  overrides: Partial<MeasuredUsage> = {},
): MeasuredUsage {
  return {
    state: "measured",
    job: figures(1600, "6100000000"),
    vm: { vcpus: 2, memory_bytes: "8000000000" },
    label_mark: null,
    chart: {
      started_at: at(0),
      ended_at: at(5),
      point_seconds: 2,
      points: [
        figures(1000, "2000000000"),
        figures(2000, "4000000000", 1),
        figures(500, "8000000000"),
      ],
    },
    steps: [],
    ...overrides,
  };
}
