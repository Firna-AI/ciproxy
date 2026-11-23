// The reader of the CPU and memory of one job, which the job page gives
// beside the job. A running job and a job without figures hold their state
// alone. A measured job keeps each null of the API apart from zero: macOS
// gives no out-of-memory count, the label mark names only the sizes that the
// VM exceeds, a step can lack a time, and a step that holds no point has no
// figures. The size of the VM and the length of a point are the units of the
// chart, so each is at least 1, and the chart holds a point and a span.

import type {
  JobUsageChart,
  JobUsageFigures,
  JobUsageLabelMark,
  JobUsageStep,
  JobUsageView,
  JobUsageVmSize,
} from "./jobs-types.ts";
import {
  array,
  boolean,
  integer,
  nullable,
  object,
  ParseError,
  string,
  time,
} from "./parse-core.ts";

/** A whole number from a lower bound, such as a count from zero. */
function wholeFrom(least: number): (value: unknown) => number {
  return (value) => {
    const number = integer(value);
    if (number < least) throw new ParseError("whole number");
    return number;
  };
}

const count = wholeFrom(0);
const positive = wholeFrom(1);

/** A byte count as a decimal string, from zero. */
function bytes(value: unknown): string {
  const text = string(value);
  if (!/^\d+$/.test(text)) throw new ParseError("byte count");
  return text;
}

/** A byte count that is more than zero, such as the memory of a VM. */
function someBytes(value: unknown): string {
  const text = bytes(value);
  if (/^0+$/.test(text)) throw new ParseError("byte count");
  return text;
}

/** Exact decimal seconds from zero, such as "2" or "0.5", kept as text. */
function seconds(value: unknown): string {
  const text = string(value);
  if (!/^\d+(\.\d+)?$/.test(text)) throw new ParseError("seconds");
  return text;
}

function figures(value: unknown): JobUsageFigures {
  const fields = object(value, "usage figures");
  return {
    average_cpu_milli_vcpu: count(fields.average_cpu_milli_vcpu),
    peak_memory_bytes: bytes(fields.peak_memory_bytes),
    oom_kills: nullable(fields.oom_kills, count),
  };
}

function vmSize(value: unknown): JobUsageVmSize {
  const fields = object(value, "usage VM size");
  return {
    vcpus: positive(fields.vcpus),
    memory_bytes: someBytes(fields.memory_bytes),
  };
}

function labelMark(value: unknown): JobUsageLabelMark {
  const fields = object(value, "label mark");
  return {
    vcpus: nullable(fields.vcpus, positive),
    memory_bytes: nullable(fields.memory_bytes, someBytes),
  };
}

function chart(value: unknown): JobUsageChart {
  const fields = object(value, "usage chart");
  const started = time(fields.started_at);
  const ended = time(fields.ended_at);
  const points = array(fields.points, figures);
  if (points.length === 0 || Date.parse(ended) <= Date.parse(started))
    throw new ParseError("usage chart");
  return {
    started_at: started,
    ended_at: ended,
    point_seconds: positive(fields.point_seconds),
    points,
  };
}

function step(value: unknown): JobUsageStep {
  const fields = object(value, "usage step");
  return {
    number: positive(fields.number),
    name: string(fields.name),
    script: boolean(fields.script),
    skipped: boolean(fields.skipped),
    started_at: nullable(fields.started_at, time),
    ended_at: nullable(fields.ended_at, time),
    duration_seconds: nullable(fields.duration_seconds, seconds),
    figures: nullable(fields.figures, figures),
  };
}

export function readJobUsage(value: unknown): JobUsageView {
  const fields = object(value, "usage");
  switch (fields.state) {
    case "running":
    case "unavailable":
      return { state: fields.state };
    case "measured":
      return {
        state: "measured",
        job: figures(fields.job),
        vm: vmSize(fields.vm),
        label_mark: nullable(fields.label_mark, labelMark),
        chart: chart(fields.chart),
        steps: array(fields.steps, step),
      };
    default:
      throw new ParseError("usage");
  }
}
