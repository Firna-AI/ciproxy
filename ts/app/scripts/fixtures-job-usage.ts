// The CPU and memory of the fixture jobs, as GET jobs/{job_id} gives them.
// The figures come from a profile of the steps of each job: a CPU level in
// vCPU and a memory ramp in GB for each step, with a small fixed ripple, one
// point each second. The chart, the steps, and the figures of the job then
// follow from the points as the API computes them: a point belongs to the
// step that holds its middle, and a chart of more than 600 points merges
// neighbours in powers of two. Build and test ran on a machine, Browser
// checks on a Boat size larger than its label, Lint ran out of memory, the
// job with unknown timing has no steps, and the older job ran on a Mac. A
// running job has no figures yet, and every other job has none.

import type { components } from "../../api/index.ts";
import { fixtureJobIds } from "./fixtures-jobs.ts";

type Schemas = components["schemas"];
type JobView = Schemas["JobView"];
type Usage = Schemas["JobUsageView"];
type Figures = Schemas["JobUsageFigures"];

/**
 * One step: its name as the API keeps it, its seconds, its CPU level in
 * vCPU, and its memory at its start and its end in GB. A step of 0 seconds
 * has its seconds alone, and a skipped step its name alone.
 */
type StepProfile =
  | readonly [
      name: string,
      seconds: number,
      cpu: number,
      from: number,
      to: number,
    ]
  | readonly [name: string, seconds: 0]
  | readonly [name: string];

/** A kill: its second, and the CPU and memory after it until the step ends. */
type Kill = { second: number; cpu: number; memoryGb: number };

type Options = { macos?: true; kill?: Kill; noSteps?: true };

type Point = { cpu: number; memoryMib: number; kills: number };

const mib = 1_048_576;
const scriptNames = ["Run script", "Pre Run script", "Post Run script"];

/** A fixed ripple between -1 and 1, the same in each run. */
function ripple(second: number, seed: number): number {
  const x = Math.sin(second * 12.9898 + seed * 78.233) * 43758.5453;
  return 2 * (x - Math.floor(x)) - 1;
}

function within(value: number, low: number, high: number): number {
  return Math.round(Math.min(Math.max(value, low), high));
}

function figuresOf(points: readonly Point[], macos: boolean): Figures | null {
  if (points.length === 0) return null;
  const cpu = points.reduce((sum, point) => sum + point.cpu, 0);
  const kills = points.reduce((sum, point) => sum + point.kills, 0);
  return {
    average_cpu_milli_vcpu: Math.floor(cpu / points.length),
    peak_memory_bytes: String(
      Math.max(...points.map((point) => point.memoryMib)) * mib,
    ),
    oom_kills: macos ? null : kills,
  };
}

function at(start: number, seconds: number): string {
  return new Date(start + seconds * 1000).toISOString();
}

/** The label sizes that the VM exceeds, as the API marks them. */
function labelMark(job: JobView): Schemas["JobUsageLabelMark"] | null {
  const vm = job.actual_size;
  const { vcpus, memory_gb: memoryGb } = job.label;
  if (vm === null || vm === undefined || vcpus === null || memoryGb === null)
    return null;
  const memory = BigInt(memoryGb) * 1_000_000_000n;
  const mark = {
    vcpus: vcpus < vm.vcpus ? vcpus : null,
    memory_bytes: memory < BigInt(vm.memory_bytes) ? String(memory) : null,
  };
  return mark.vcpus === null && mark.memory_bytes === null ? null : mark;
}

function measured(
  job: JobView,
  startedAt: string,
  profile: readonly StepProfile[],
  options: Options = {},
): Usage {
  const vm = job.actual_size;
  if (vm === null || vm === undefined) throw new Error(`No VM: ${job.name}`);
  const start = Date.parse(startedAt);
  const macos = options.macos === true;
  const points: Point[] = [];
  const spans: ([number, number] | null)[] = [];
  for (const step of profile) {
    if (step.length === 1) {
      spans.push(null);
      continue;
    }
    const [cpu, from, to] =
      step.length === 5 ? [step[2], step[3], step[4]] : [0, 0, 0];
    const first = points.length;
    const end = first + step[1];
    const kill =
      options.kill !== undefined &&
      options.kill.second >= first &&
      options.kill.second < end
        ? options.kill
        : undefined;
    const rampEnd = kill === undefined ? end : kill.second + 1;
    for (let second = first; second < end; second += 1) {
      const after = kill !== undefined && second > kill.second;
      const share = (second - first + 0.5) / (rampEnd - first);
      const busy = after ? kill.cpu : cpu;
      const used = after ? kill.memoryGb : from + (to - from) * share;
      const wave = Math.sin((second + job.name.length) / 9);
      points.push({
        cpu: within(
          busy * 1000 * (1 + 0.1 * ripple(second, 1) + 0.06 * wave),
          20,
          vm.vcpus * 1000,
        ),
        memoryMib: within(
          ((used * 1e9) / mib) * (1 + 0.01 * ripple(second, 2) + 0.015 * wave),
          100,
          (Number(vm.memory_bytes) / mib) * 0.975,
        ),
        kills: kill !== undefined && second === kill.second ? 1 : 0,
      });
    }
    spans.push([first, end]);
  }
  let width = 1;
  while (Math.ceil(points.length / width) > 600) width *= 2;
  const chart: Figures[] = [];
  for (let index = 0; index < points.length; index += width) {
    const group = figuresOf(points.slice(index, index + width), macos);
    if (group !== null) chart.push(group);
  }
  const whole = figuresOf(points, macos);
  if (whole === null) throw new Error(`No points: ${job.name}`);
  return {
    state: "measured",
    job: whole,
    vm: { vcpus: vm.vcpus, memory_bytes: vm.memory_bytes },
    label_mark: labelMark(job),
    chart: {
      started_at: at(start, 0),
      ended_at: at(start, points.length),
      point_seconds: width,
      points: chart,
    },
    steps: options.noSteps
      ? []
      : profile.map((step, index) => {
          const span = spans[index] ?? null;
          return {
            number: index + 1,
            name: step[0],
            script: scriptNames.includes(step[0]),
            skipped: span === null,
            started_at: span === null ? null : at(start, span[0]),
            ended_at: span === null ? null : at(start, span[1]),
            duration_seconds: span === null ? null : String(span[1] - span[0]),
            figures:
              span === null
                ? null
                : figuresOf(points.slice(span[0], span[1]), macos),
          };
        }),
  };
}

const profiles: Readonly<Record<string, (job: JobView) => Usage>> = {
  [fixtureJobIds.machine]: (job) =>
    measured(job, "2026-11-06T11:50:00Z", [
      ["Set up job", 2, 0.3, 0.42, 0.48],
      ["Run actions/checkout@v4", 2, 0.4, 0.5, 0.55],
      ["Run actions/setup-node@v4", 5, 0.9, 0.6, 0.8],
      ["Run script", 41, 2.2, 0.9, 2.1],
      ["Build", 128, 3.6, 2.2, 4.6],
      ["Test", 185, 2.9, 3.8, 6.2],
      ["Post Run actions/setup-node@v4", 8, 0.8, 1.9, 1.8],
      ["Post Run actions/checkout@v4", 0],
      ["Complete job", 1, 0.2, 1.0, 1.0],
    ]),
  [fixtureJobIds.boat]: (job) =>
    measured(job, "2026-11-06T11:50:00Z", [
      ["Set up job", 3, 0.6, 0.7, 0.75],
      ["Run actions/checkout@v4", 2, 0.5, 0.75, 0.8],
      ["Run actions/setup-node@v4", 6, 1.1, 0.8, 1.0],
      ["Run script", 38, 2.6, 1.1, 2.3],
      ["Install Playwright browsers", 41, 1.8, 2.3, 2.7],
      ["Build", 92, 6.4, 2.8, 4.9],
      ["Test", 186, 4.6, 4.2, 7.4],
      ["Post Run actions/setup-node@v4", 3, 0.9, 1.9, 1.8],
      ["Post Run actions/checkout@v4", 0],
      ["Complete job", 1, 0.3, 1.2, 1.2],
    ]),
  [fixtureJobIds.failed]: (job) =>
    measured(
      job,
      "2026-11-06T11:50:00Z",
      [
        ["Set up job", 2, 0.3, 0.42, 0.48],
        ["Run actions/checkout@v4", 2, 0.4, 0.5, 0.55],
        ["Run actions/setup-node@v4", 5, 0.8, 0.6, 0.8],
        ["Run script", 39, 1.9, 0.9, 2.2],
        ["Lint all packages", 296, 3.4, 2.4, 8.2],
        ["Upload lint report", 18, 0.5, 1.7, 1.6],
        ["Post Run actions/setup-node@v4"],
        ["Post Run actions/checkout@v4", 9, 0.2, 1.4, 1.4],
        ["Complete job", 1, 0.2, 1.2, 1.2],
      ],
      { kill: { second: 328, cpu: 0.5, memoryGb: 2.4 } },
    ),
  [fixtureJobIds.untimed]: (job) =>
    measured(
      job,
      "2026-11-06T11:30:00Z",
      [
        ["Set up job", 2, 0.3, 0.42, 0.48],
        ["Run actions/checkout@v4", 3, 0.4, 0.5, 0.55],
        ["Run script", 96, 2.1, 0.9, 2.4],
        ["Test", 1120, 3.1, 2.5, 5.6],
        ["Post Run actions/checkout@v4", 12, 0.3, 1.2, 1.2],
        ["Complete job", 1, 0.2, 1.0, 1.0],
      ],
      { noSteps: true },
    ),
  [fixtureJobIds.older]: (job) =>
    measured(
      job,
      "2026-10-10T11:50:00Z",
      [
        ["Set up job", 2, 0.4, 2.1, 2.15],
        ["Run actions/checkout@v4", 3, 0.5, 2.2, 2.25],
        ["Run actions/setup-node@v4", 5, 0.9, 2.3, 2.5],
        ["Run script", 46, 1.4, 2.5, 3.3],
        ["Build", 118, 2.9, 3.4, 5.2],
        ["Test", 188, 2.3, 4.6, 6.8],
        ["Post Run actions/setup-node@v4", 4, 0.8, 3.1, 3.0],
        ["Post Run actions/checkout@v4", 5, 0.3, 2.8, 2.8],
        ["Complete job", 1, 0.3, 2.6, 2.6],
      ],
      { macos: true },
    ),
};

/** The usage that the job page of a fixture job gives. */
export function fixtureUsage(job: JobView): Usage {
  if (job.status === "running") return { state: "running" };
  return profiles[job.id]?.(job) ?? { state: "unavailable" };
}
