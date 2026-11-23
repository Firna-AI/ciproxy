// The CPU and memory of a job, as the job page shows them. The guest program
// samples the whole job VM once a second, from the start of the job to the
// exit of the runner, and reads the name, the start, and the end of each step
// from the runner's log. The time before the first step shows as Set up job,
// and the time after the last step as Complete job, as on GitHub. A name that
// starts with "Run ", "Pre Run ", or "Post Run " can come from a run command,
// so ciproxy keeps it only when the rest names a remote action; each other
// such name shows as Run script, Pre Run script, or Post Run script. A name
// such as Running tests keeps its text. A point belongs to the step that
// holds its middle, so a step of under a second can hold no point and shows
// no figures, as a skipped step does. macOS counts no out-of-memory kills.
// The figures come once, when the job ends. On a VM that is larger than the
// size of the label, such as a Boat size, the figures also name the label
// size.
//
// The fixtures build the points of a job from a profile of its steps: a CPU
// level and a memory ramp, with a small fixed ripple. The step figures then
// come from the points, as the API computes them, so the chart and the table
// agree.

import type { JobDetail } from "./job-details.ts";
import type { Platform } from "./platform.ts";

/** The vCPU and memory of the VM that ran a job: the top of its figures. */
export type UsageVm = { vcpu: number; memoryGb: number };

/** The figures of the points of a step or of a whole job. */
export type UsageFigures = {
  averageCpu: number;
  peakMemoryGb: number;
  /**
   * The processes that the kernel stopped for lack of memory, or null on
   * macOS, which gives no such count.
   */
  oomKills: number | null;
};

export type UsageStep = {
  /** The number of the step on GitHub, from 1 for Set up job. */
  number: number;
  /** The name that ciproxy keeps. */
  name: string;
  /** The name hides a script, so the row links to the step on GitHub. */
  script: boolean;
  /** The seconds from the start of the job, or null for a skipped step. */
  span: { start: number; end: number } | null;
  /** The figures of the points of the step, or null when it holds none. */
  figures: UsageFigures | null;
};

export type MeasuredUsage = {
  state: "measured";
  vm: UsageVm;
  /**
   * The vCPU and memory of the label, when the VM that ran the job is larger
   * in one of them, as a Boat size can be; null when they are equal or the
   * request of the job is unknown.
   */
  labelSize: UsageVm | null;
  /** The length of a point in seconds. */
  pointSeconds: number;
  /** The busy vCPU of each point. */
  cpu: readonly number[];
  /** The memory in use at the peak of each point, in GB. */
  memoryGb: readonly number[];
  /** The out-of-memory kills of each point; null on macOS. */
  oomKills: readonly number[] | null;
  /** The steps, or null when ciproxy could not read them. */
  steps: readonly UsageStep[] | null;
  job: UsageFigures;
};

/**
 * The figures of a job; a running job, whose figures come when it ends; or a
 * job without figures, as from an older agent or image.
 */
export type JobUsage =
  MeasuredUsage | { state: "running" } | { state: "unavailable" };

/**
 * A remote action as GitHub names it: owner/repo@ref, owner/repo/path@ref, or
 * docker://image. An owner starts with a letter or a digit, so a local path
 * such as ./deploy@prod is not one.
 */
const remoteAction =
  /^(?:[A-Za-z0-9][A-Za-z0-9-]*\/[\w.-]+(?:\/[^\s@]+)*@\S+|docker:\/\/\S+)$/;

/**
 * The name that ciproxy keeps for a step name of the runner's log, and
 * whether it hides a script. Only a name that starts with "Run ", "Pre Run ",
 * or "Post Run " can come from a run command.
 */
export function keptStepName(name: string): { name: string; script: boolean } {
  const candidate = /^((?:Pre |Post )?Run) (.*)$/s.exec(name);
  if (!candidate || remoteAction.test((candidate[2] ?? "").trim())) {
    return { name, script: false };
  }
  return { name: `${candidate[1]} script`, script: true };
}

/**
 * One step of a profile: its name as the runner's log gives it, its seconds,
 * its CPU level in vCPU, and its memory at its start and its end, in GB. A
 * step of 0 seconds has its seconds alone, and a skipped step its name alone.
 */
export type StepProfile =
  | readonly [
      name: string,
      seconds: number,
      cpu: number,
      from: number,
      to: number,
    ]
  | readonly [name: string, seconds: 0]
  | readonly [name: string];

/**
 * The out-of-memory kill of a job: its second, and the CPU and memory after
 * it until its step ends.
 */
export type KillProfile = { second: number; cpu: number; memoryGb: number };

type ProfileOptions = {
  platform?: Platform;
  kill?: KillProfile;
  /** ciproxy could not read the steps, so the job has its points alone. */
  noSteps?: true;
};

/**
 * The share of its memory that a VM leaves to its job. The kernel of the VM
 * keeps the rest, so a job runs out of memory under the VM size.
 */
const usableMemory = 0.975;

/** A fixed ripple between -1 and 1, the same in each build. */
function ripple(second: number, seed: number): number {
  const x = Math.sin(second * 12.9898 + seed * 78.233) * 43758.5453;
  return 2 * (x - Math.floor(x)) - 1;
}

function within(value: number, low: number, high: number): number {
  return Math.round(Math.min(Math.max(value, low), high) * 100) / 100;
}

/** "6m 12s" -> 372, the duration of a job in seconds. */
function durationSeconds(text: string): number {
  const match = /^(?:(\d+)m )?(\d+)s$/.exec(text);
  if (!match) throw new Error(`No duration in ${text}`);
  return Number(match[1] ?? 0) * 60 + Number(match[2]);
}

/** The VM that ran a job: the Boat size, the request, or the VM it reported. */
function vmOf(job: JobDetail): UsageVm {
  const size =
    job.boatSize ?? (job.request === "unknown" ? job.vm : job.request);
  if (!size) throw new Error(`No VM for ${job.name}`);
  return { vcpu: size.vcpu, memoryGb: size.memoryGb };
}

/** The size of the label, when the VM that ran the job is larger. */
function labelSizeOf(job: JobDetail, vm: UsageVm): UsageVm | null {
  if (job.request === "unknown") return null;
  const { vcpu, memoryGb } = job.request;
  return vcpu < vm.vcpu || memoryGb < vm.memoryGb ? { vcpu, memoryGb } : null;
}

type Points = Pick<
  MeasuredUsage,
  "pointSeconds" | "cpu" | "memoryGb" | "oomKills"
>;

/** The figures of the points whose middle lies in a span; null for none. */
function figuresOf(
  points: Points,
  start: number,
  end: number,
): UsageFigures | null {
  const held = points.cpu
    .map((_, i) => i)
    .filter((i) => {
      const middle = (i + 0.5) * points.pointSeconds;
      return middle >= start && middle < end;
    });
  if (held.length === 0) return null;
  const sum = (values: readonly number[]) =>
    held.reduce((total, i) => total + (values[i] ?? 0), 0);
  return {
    averageCpu: sum(points.cpu) / held.length,
    peakMemoryGb: Math.max(...held.map((i) => points.memoryGb[i] ?? 0)),
    oomKills: points.oomKills ? sum(points.oomKills) : null,
  };
}

/** The figures of a job from the profile of its steps. */
export function measured(
  job: JobDetail,
  profile: readonly StepProfile[],
  options: ProfileOptions = {},
): MeasuredUsage {
  const vm = vmOf(job);
  const seed = job.name.length;
  const { kill } = options;
  const cpu: number[] = [];
  const memoryGb: number[] = [];
  const kills: number[] = [];
  const steps: Omit<UsageStep, "figures">[] = [];
  let start = 0;
  profile.forEach((step, index) => {
    const base = { number: index + 1, ...keptStepName(step[0]) };
    if (step.length === 1) {
      steps.push({ ...base, span: null });
      return;
    }
    const [, seconds, level, from, to] =
      step.length === 2 ? ([step[0], 0, 0, 0, 0] as const) : step;
    const end = start + seconds;
    const stop =
      kill && kill.second >= start && kill.second < end ? kill : undefined;
    const rampEnd = stop ? stop.second + 1 : end;
    for (let second = start; second < end; second += 1) {
      const after = stop && second > stop.second ? stop : undefined;
      const share = (second - start + 0.5) / (rampEnd - start);
      const busy = after ? after.cpu : level;
      const used = after ? after.memoryGb : from + (to - from) * share;
      const wave = Math.sin((second + seed) / 9);
      const cpuRipple = 1 + 0.1 * ripple(second, seed) + 0.06 * wave;
      const memoryRipple = 1 + 0.01 * ripple(second, seed + 1) + 0.015 * wave;
      cpu.push(within(busy * cpuRipple, 0.02, vm.vcpu));
      memoryGb.push(
        within(used * memoryRipple, 0.1, vm.memoryGb * usableMemory),
      );
      kills.push(stop && second === stop.second ? 1 : 0);
    }
    steps.push({ ...base, span: { start, end } });
    start = end;
  });
  if (start !== durationSeconds(job.duration)) {
    throw new Error(
      `The steps of ${job.name} last ${start}s, not ${job.duration}.`,
    );
  }
  const points: Points = {
    pointSeconds: 1,
    cpu,
    memoryGb,
    oomKills: options.platform === "macos" ? null : kills,
  };
  const figures = (span: UsageStep["span"]) =>
    span ? figuresOf(points, span.start, span.end) : null;
  const whole = figuresOf(points, 0, start);
  if (!whole) throw new Error(`No points for ${job.name}`);
  return {
    state: "measured",
    vm,
    labelSize: labelSizeOf(job, vm),
    ...points,
    steps: options.noSteps
      ? null
      : steps.map((step) => ({ ...step, figures: figures(step.span) })),
    job: whole,
  };
}
