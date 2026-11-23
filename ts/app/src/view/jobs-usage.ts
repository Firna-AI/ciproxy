// The CPU and memory panel of the job page as a model. A finished job shows
// its average CPU and its peak memory against the VM that ran it, with the
// size of the label when the VM is larger, and on Linux its out-of-memory
// kills. Each step shows its duration and its figures, and a step whose name
// hides a script links to that step on GitHub. A running job and a job
// without figures say so in one sentence, and a job that no runner took has
// no panel. A measured figure has one decimal, and a VM size or a label size
// reads as the Details panel writes it.

import type {
  JobUsageFigures,
  JobUsageStep,
  JobUsageView,
  JobView,
} from "../api/jobs-types.ts";
import { dotJoin } from "./exact-format.ts";
import {
  countText,
  formatMeasuredGigabytes,
  formatMeasuredVcpu,
} from "./format.ts";
import { durationText, memoryText, placeOf, vcpuText } from "./jobs-format.ts";

export type MeasuredUsage = Extract<JobUsageView, { state: "measured" }>;

export const usageCopy = {
  title: "CPU and memory",
  running: "The figures show when the job ends.",
  unavailable: "Figures are not available for this job.",
  noSteps: "Figures for each step are not available for this job.",
  wholeVm:
    "Each figure covers the whole VM, also the services and processes that earlier steps started.",
  skipped: "Skipped",
  noFigures: "No figures",
} as const;

/** A share of the VM: the fill of a meter, with the label size as a tick. */
export type UsageMeter = { share: number; mark: number | null };

export type UsageFigure = {
  label: string;
  value: string;
  /** The size of the VM, and of the label when the VM is larger. */
  caption: string | null;
  meter: UsageMeter | null;
  /** The value counts processes that ran out of memory. */
  alert: boolean;
};

export type UsageStepFigures = {
  cpu: string;
  memory: string;
  /** "1 process", only when the step ran out of memory. */
  kills: string | null;
};

export type UsageStepRow = {
  number: number;
  name: string;
  /** The step on GitHub, for a name that hides a script. */
  href: string | null;
  duration: string;
  /** The step has a duration. Skipped and Unknown read muted. */
  timed: boolean;
  figures: UsageStepFigures | null;
};

export type UsagePanelModel =
  | {
      state: "measured";
      usage: MeasuredUsage;
      figures: readonly UsageFigure[];
      /** The steps, or null when ciproxy could not read them. */
      steps: readonly UsageStepRow[] | null;
      /** A step ran out of memory, so the table shows that column. */
      killColumn: boolean;
    }
  | { state: "running" | "unavailable"; text: string };

/** "1 process", "2 processes": the processes that ran out of memory. */
export function processes(count: number): string {
  return countText(count, "process", "processes");
}

/** The share of a whole, from 0 to 1. */
function share(part: number, whole: number): number {
  return Math.min(Math.max(part / whole, 0), 1);
}

/** "of 8 vCPU", or "of 8 vCPU · label 2 vCPU" when the label is smaller. */
function ofText(vm: string, label: string | null): string {
  return label === null ? `of ${vm}` : dotJoin(`of ${vm}`, `label ${label}`);
}

function jobFigures(usage: MeasuredUsage): UsageFigure[] {
  const { job, vm } = usage;
  const cpuMark = usage.label_mark?.vcpus ?? null;
  const memoryMark = usage.label_mark?.memory_bytes ?? null;
  const memory = Number(vm.memory_bytes);
  const figures: UsageFigure[] = [
    {
      label: "Average CPU",
      value: formatMeasuredVcpu(job.average_cpu_milli_vcpu),
      caption: ofText(
        vcpuText(vm.vcpus),
        cpuMark === null ? null : vcpuText(cpuMark),
      ),
      meter: {
        share: share(job.average_cpu_milli_vcpu, vm.vcpus * 1000),
        mark: cpuMark === null ? null : share(cpuMark, vm.vcpus),
      },
      alert: false,
    },
    {
      label: "Peak memory",
      value: formatMeasuredGigabytes(job.peak_memory_bytes),
      caption: ofText(
        memoryText(vm.memory_bytes),
        memoryMark === null ? null : memoryText(memoryMark),
      ),
      meter: {
        share: share(Number(job.peak_memory_bytes), memory),
        mark: memoryMark === null ? null : share(Number(memoryMark), memory),
      },
      alert: false,
    },
  ];
  if (job.oom_kills !== null) {
    figures.push({
      label: "Out of memory",
      value: job.oom_kills === 0 ? "None" : processes(job.oom_kills),
      caption: null,
      meter: null,
      alert: job.oom_kills > 0,
    });
  }
  return figures;
}

/** The log of one step on GitHub: the page of the job, open at the step. */
export function stepHref(
  githubUrl: string | null,
  step: number,
): string | null {
  if (githubUrl === null) return null;
  try {
    const url = new URL(githubUrl);
    url.hash = `step:${step}:1`;
    return url.href;
  } catch {
    return null;
  }
}

function stepFigures(figures: JobUsageFigures): UsageStepFigures {
  const kills = figures.oom_kills ?? 0;
  return {
    cpu: formatMeasuredVcpu(figures.average_cpu_milli_vcpu),
    memory: formatMeasuredGigabytes(figures.peak_memory_bytes),
    kills: kills > 0 ? processes(kills) : null,
  };
}

function stepRow(step: JobUsageStep, githubUrl: string | null): UsageStepRow {
  return {
    number: step.number,
    name: step.name,
    href: step.script ? stepHref(githubUrl, step.number) : null,
    duration: step.skipped
      ? usageCopy.skipped
      : durationText(step.duration_seconds),
    timed: !step.skipped && step.duration_seconds !== null,
    figures: step.figures === null ? null : stepFigures(step.figures),
  };
}

/** "1.9 vCPU average · 5.2 GB peak": the figures of a step on one line. */
export function figureLine(figures: UsageStepFigures): string {
  return dotJoin(`${figures.cpu} average`, `${figures.memory} peak`);
}

export function usagePanelModel(
  job: JobView,
  usage: JobUsageView,
): UsagePanelModel | null {
  if (placeOf(job.location, job.counts_as_start).kind === "none") return null;
  if (usage.state === "running")
    return { state: "running", text: usageCopy.running };
  if (usage.state === "unavailable")
    return { state: "unavailable", text: usageCopy.unavailable };
  const steps = usage.steps.map((step) =>
    stepRow(step, job.github_url ?? null),
  );
  return {
    state: "measured",
    usage,
    figures: jobFigures(usage),
    steps: steps.length === 0 ? null : steps,
    killColumn: steps.some(
      (row) => row.figures !== null && row.figures.kills !== null,
    ),
  };
}
