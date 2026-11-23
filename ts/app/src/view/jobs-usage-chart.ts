// The CPU and memory of a job over time, as the geometry of the chart of the
// job page. Two plots share one time axis, CPU above memory, because the two
// figures have different units. The top of each plot is the vCPU or the
// memory of the VM, and each plot draws in a box of the seconds of the chart
// across and 0 to 100 down, which stretches to the width of the panel. A
// lane marks each step that lasted, a hairline marks the start of each later
// step, a dot marks each out-of-memory kill on the memory plot, and a line
// marks the size of the label where the VM is larger. A point lies at the
// middle of its time, and the last point can be shorter than the others.

import type { JobUsageChart, JobUsageStep } from "../api/jobs-types.ts";
import { dotJoin } from "./exact-format.ts";
import {
  formatMeasuredGigabytes,
  formatMeasuredVcpu,
  wholeGigabytes,
} from "./format.ts";
import { memoryText, secondsText, vcpuText } from "./jobs-format.ts";
import type { MeasuredUsage } from "./jobs-usage.ts";

/** A label of an axis, at a share of the axis from its start. */
export type AxisTick = { text: string; share: number };

export type ChartPlot = {
  title: string;
  unit: string;
  /** The top, the middle, and the base, as shares from the bottom. */
  ticks: readonly AxisTick[];
  line: string;
  area: string;
  /** The height of the label line in the box, 0 to 100 down, or null. */
  label: number | null;
  /** Each out-of-memory kill, as shares of the plot from its top left. */
  kills: readonly { left: number; top: number }[];
};

/** A step that lasted, as shares of the lane. */
export type LaneBlock = { number: number; left: number; width: number };

export type UsageChartModel = {
  /** The seconds of the chart, the width of the box of each plot. */
  seconds: number;
  /** The figures of the job in words, for screen readers. */
  summary: string;
  /** The steps that lasted, or null when the job has no steps. */
  lane: readonly LaneBlock[] | null;
  /** The start of each step after the first, in seconds. */
  starts: readonly number[];
  plots: readonly [ChartPlot, ChartPlot];
  times: readonly AxisTick[];
  /** "Label size: 2 vCPU · 8 GB", when the label is smaller than the VM. */
  labelKey: string | null;
  killKey: boolean;
};

/** The spaces between time ticks, in seconds, shortest first. */
const tickSpaces = [
  5, 10, 15, 30, 60, 120, 300, 600, 900, 1800, 3600, 7200, 10800, 21600, 43200,
  86400,
];

const day = 86_400;

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

/** 372 -> "6:12", 3720 -> "1:02:00": the time from the first sample. */
export function clockText(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = pad(Math.floor(seconds % 60));
  return hours > 0 ? `${hours}:${pad(minutes)}:${rest}` : `${minutes}:${rest}`;
}

/** The time ticks: at most 4 spaces on a phone and 8 on a wider window. */
function timeTicks(seconds: number, mobile: boolean): AxisTick[] {
  const most = mobile ? 4 : 8;
  const space =
    tickSpaces.find((value) => seconds / value <= most) ??
    Math.ceil(seconds / most / day) * day;
  return Array.from(
    { length: Math.floor(seconds / space) + 1 },
    (_, index) => ({
      text: clockText(index * space),
      share: (index * space) / seconds,
    }),
  );
}

/** A value of an axis, whole or to one place: 4 -> "4", 1.5 -> "1.5". */
function tickText(value: number): string {
  const rounded = Math.round(value * 10) / 10;
  return Number.isInteger(rounded) ? String(rounded) : rounded.toFixed(1);
}

function within(share: number): number {
  return Math.min(Math.max(share, 0), 1);
}

/** A number of the box, to two places. */
function coordinate(value: number): string {
  return String(Math.round(value * 100) / 100);
}

/** The middle of each point, in seconds from the start of the chart. */
function middles(chart: JobUsageChart, seconds: number): number[] {
  return chart.points.map((_, index) => {
    const start = Math.min(index * chart.point_seconds, seconds);
    const end = Math.min(start + chart.point_seconds, seconds);
    return (start + end) / 2;
  });
}

/** The line and the area under it, from the left edge to the right edge. */
function shapes(
  xs: readonly number[],
  shares: readonly number[],
  seconds: number,
): Pick<ChartPlot, "line" | "area"> {
  const ys = shares.map((value) => coordinate(100 - within(value) * 100));
  const run = xs.map((x, index) => `L${coordinate(x)},${ys[index]}`).join("");
  const end = coordinate(seconds);
  const first = ys[0] ?? "100";
  const last = ys.at(-1) ?? "100";
  return {
    line: `M0,${first}${run}L${end},${last}`,
    area: `M0,100L0,${first}${run}L${end},${last}L${end},100Z`,
  };
}

function plots(usage: MeasuredUsage, seconds: number): [ChartPlot, ChartPlot] {
  const { chart, vm } = usage;
  const xs = middles(chart, seconds);
  const memory = Number(vm.memory_bytes);
  const cpuShares = chart.points.map(
    (point) => point.average_cpu_milli_vcpu / (vm.vcpus * 1000),
  );
  const memoryShares = chart.points.map(
    (point) => Number(point.peak_memory_bytes) / memory,
  );
  const cpuLabel = usage.label_mark?.vcpus ?? null;
  const memoryLabel = usage.label_mark?.memory_bytes ?? null;
  return [
    {
      title: "CPU",
      unit: "vCPU",
      ticks: [
        { text: String(vm.vcpus), share: 1 },
        { text: tickText(vm.vcpus / 2), share: 0.5 },
        { text: "0", share: 0 },
      ],
      ...shapes(xs, cpuShares, seconds),
      label: cpuLabel === null ? null : 100 - within(cpuLabel / vm.vcpus) * 100,
      kills: [],
    },
    {
      title: "Memory",
      unit: "GB",
      ticks: [
        { text: String(wholeGigabytes(vm.memory_bytes)), share: 1 },
        { text: tickText(memory / 2e9), share: 0.5 },
        { text: "0", share: 0 },
      ],
      ...shapes(xs, memoryShares, seconds),
      label:
        memoryLabel === null
          ? null
          : 100 - within(Number(memoryLabel) / memory) * 100,
      kills: chart.points.flatMap((point, index) =>
        (point.oom_kills ?? 0) > 0
          ? [
              {
                left: (xs[index] ?? 0) / seconds,
                top: 1 - within(memoryShares[index] ?? 0),
              },
            ]
          : [],
      ),
    },
  ];
}

/** The time of a step in seconds from the start, within the chart. */
function span(
  step: JobUsageStep,
  start: number,
  seconds: number,
): { from: number; to: number } | null {
  if (step.started_at === null || step.ended_at === null) return null;
  const from = (Date.parse(step.started_at) - start) / 1000;
  const to = (Date.parse(step.ended_at) - start) / 1000;
  if (to < from) return null;
  const clamp = (value: number) => Math.min(Math.max(value, 0), seconds);
  return { from: clamp(from), to: clamp(to) };
}

function summary(usage: MeasuredUsage, seconds: number, label: string[]) {
  const { job, vm } = usage;
  const sizes =
    label.length === 0 ? "" : ` The label gives ${label.join(" and ")}.`;
  return `CPU and memory over ${secondsText(Math.round(seconds))}: average CPU ${formatMeasuredVcpu(job.average_cpu_milli_vcpu)} of ${vcpuText(vm.vcpus)}, peak memory ${formatMeasuredGigabytes(job.peak_memory_bytes)} of ${memoryText(vm.memory_bytes)}.${sizes}`;
}

export function usageChartModel(
  usage: MeasuredUsage,
  mobile: boolean,
): UsageChartModel {
  const start = Date.parse(usage.chart.started_at);
  const seconds = (Date.parse(usage.chart.ended_at) - start) / 1000;
  const spans = usage.steps.flatMap((step) => {
    const time = span(step, start, seconds);
    return time === null ? [] : [{ number: step.number, ...time }];
  });
  const mark = usage.label_mark;
  const label = [
    mark?.vcpus == null ? null : vcpuText(mark.vcpus),
    mark?.memory_bytes == null ? null : memoryText(mark.memory_bytes),
  ].filter((part) => part !== null);
  return {
    seconds,
    summary: summary(usage, seconds, label),
    lane:
      usage.steps.length === 0
        ? null
        : spans
            .filter((time) => time.to > time.from)
            .map((time) => ({
              number: time.number,
              left: time.from / seconds,
              width: (time.to - time.from) / seconds,
            })),
    starts: [...new Set(spans.map((time) => time.from))]
      .filter((from) => from > 0 && from < seconds)
      .sort((a, b) => a - b),
    plots: plots(usage, seconds),
    times: timeTicks(seconds, mobile),
    labelKey: label.length === 0 ? null : `Label size: ${dotJoin(...label)}`,
    killKey: usage.chart.points.some((point) => (point.oom_kills ?? 0) > 0),
  };
}
