// The four steps of Setup: the GitHub App, a machine, an overflow provider,
// and the first job. Every fact comes from the API: the organizations, the
// machines that joined with their state, the provider accounts, and the
// first job with its phase and its result. A job counts as passed only when
// GitHub reports success. The provider step can be skipped, and it never
// holds Setup open: Setup is complete when the first job finished, because
// the API keeps no record of a skip. A machine that gets its first job image
// still completes its step.

import type { MachineView, SetupJob, SetupView } from "../api/schema.ts";
import {
  countText,
  formatDuration,
  formatGigabytes,
  joinWords,
} from "./format.ts";
import { imageCopy } from "./machines-copy.ts";
import { machineStatus, otherMachineTakesJobs } from "./machines.ts";
import { macJobsLine } from "./machines-capacity.ts";
import {
  canStartVms,
  defaultLabelName,
  isMac,
  virtualizationProblem,
} from "./platform.ts";
import { providerName } from "./providers.ts";

/** The most job VMs of a machine at once: 2 on a Mac, else no limit. */
function vmLimitOf(machine: MachineView): number {
  return machine.vm_limit ?? Infinity;
}

export type StepStatus = "done" | "active" | "todo" | "skipped";

export type SetupStep = {
  key: "github" | "machine" | "provider" | "job";
  title: string;
  status: StepStatus;
  /** The line of a closed step: what it did, or what it asks for. */
  summary: string;
};

export type ConnectedMachine = {
  name: string;
  facts: string;
  /** Set when the disk holds fewer jobs of ciproxy than the slots. */
  diskNote: string | null;
  /** The machine gets its first job image, so it takes no job yet. */
  preparing: boolean;
};

/** What Setup knows of the providers: the line of a connection, or a skip. */
export type ProviderProgress = {
  /** Set while a provider account with an accepted key is connected. */
  connected: string | null;
  /** Skip for now in this tab, which counts only while none is connected. */
  skipped: boolean;
};

export type FirstJob =
  | { kind: "waiting" }
  | { kind: "moving"; text: string }
  | { kind: "finished"; passed: boolean; title: string; text: string };

export type SetupModel = {
  steps: readonly SetupStep[];
  done: number;
  total: number;
  complete: boolean;
  connected: ConnectedMachine | null;
  /** A machine that joined but cannot start job VMs, with the fix. */
  cannotStartVms: { host: string; fix: string } | null;
  /** Organizations whose App GitHub suspended. */
  suspended: readonly string[];
  job: FirstJob;
  /** The machine whose first job image the waiting first job waits for. */
  imageWait: string | null;
};

const outcomes: Readonly<Record<string, string>> = {
  failure: "failed",
  cancelled: "was cancelled",
  timed_out: "timed out",
  skipped: "was skipped",
};

function placeText(job: SetupJob): string {
  if (job.place?.kind === "machine") return " on your machine";
  if (job.place?.kind === "overflow") {
    return ` on ${providerName(job.place.provider)}`;
  }
  return "";
}

export function firstJob(job: SetupJob | null | undefined): FirstJob {
  if (job === null || job === undefined) return { kind: "waiting" };
  const subject = `${job.name} in ${job.repository}`;
  switch (job.phase) {
    case "available":
      return { kind: "moving", text: `${subject} is waiting for a runner…` };
    case "assigned":
      return { kind: "moving", text: `${subject} is starting…` };
    case "started":
      return {
        kind: "moving",
        text: `${subject} is running${placeText(job)}…`,
      };
    case "completed":
      break;
  }
  const duration =
    job.started_at && job.completed_at
      ? formatDuration(job.started_at, job.completed_at)
      : null;
  if (job.result === "success") {
    const time = duration === null ? "" : ` in ${duration}`;
    return {
      kind: "finished",
      passed: true,
      title: "Your first job passed",
      text: `${subject} passed${time}${placeText(job)}.`,
    };
  }
  const verb = job.result ? (outcomes[job.result] ?? "finished") : "finished";
  return {
    kind: "finished",
    passed: false,
    title: "Your first job finished",
    text: `${subject} ${verb}${placeText(job)}. Its logs are on GitHub.`,
  };
}

export function connectedMachine(machine: MachineView): ConnectedMachine {
  const jobs = machine.concurrent_default_jobs;
  const atOnce = jobs === null || jobs === undefined ? null : Number(jobs);
  return {
    name: machine.hostname,
    facts: [
      isMac(machine)
        ? countText(machine.threads, "core", "cores")
        : countText(machine.threads, "thread", "threads"),
      formatGigabytes(machine.memory_bytes),
      isMac(machine) ? "Virtualization ready" : "KVM ready",
      countText(machine.slots, "slot", "slots"),
    ].join(" · "),
    diskNote:
      atOnce !== null && atOnce < Math.min(machine.slots, vmLimitOf(machine))
        ? `Its disk holds ${countText(atOnce, "job", "jobs")} of ${defaultLabelName(machine)} at a time.`
        : macJobsLine(machine),
    preparing: machineStatus(machine) === "preparing",
  };
}

/** "forge-01 · 6 slots", or the image wait of the machine. */
function machineSummary(machine: MachineView): string {
  const preparing = machineStatus(machine) === "preparing";
  return `${machine.hostname} · ${
    preparing ? imageCopy.step : countText(machine.slots, "slot", "slots")
  }`;
}

/**
 * The machine that the first job waits for while it gets its first job
 * image, or null. The job waits for it only while it waits for a runner, no
 * other machine takes jobs, and no provider is connected. With a provider
 * connected, routing starts the job on that provider at once.
 */
function imageWait(
  setup: SetupView,
  machine: MachineView | undefined,
  machines: readonly MachineView[],
  provider: ProviderProgress,
): string | null {
  if (setup.first_job?.phase !== "available") return null;
  if (machine === undefined || machineStatus(machine) !== "preparing")
    return null;
  if (provider.connected !== null || otherMachineTakesJobs(machine, machines))
    return null;
  return machine.hostname;
}

function jobSummary(job: FirstJob): string {
  if (job.kind !== "finished") return "Change runs-on in one workflow.";
  return job.passed ? "Your first job passed." : "Your first job finished.";
}

function providerStatus(
  provider: ProviderProgress,
  machineDone: boolean,
): StepStatus {
  if (provider.connected !== null) return "done";
  if (provider.skipped) return "skipped";
  return machineDone ? "active" : "todo";
}

/** Setup is complete when the first job finished, with or without overflow. */
export function setupComplete(
  setup: SetupView,
  machines: readonly MachineView[],
): boolean {
  return (
    setup.organizations.some((item) => item.state === "active") &&
    machines.some(canStartVms) &&
    firstJob(setup.first_job).kind === "finished"
  );
}

/**
 * The model of Setup. `holdMachine` keeps the machine step open after a
 * machine joined while the page was open, until the person continues. The
 * first job opens after the provider step, or at once when a job already
 * runs.
 */
export function setupModel(
  setup: SetupView,
  machines: readonly MachineView[],
  holdMachine: boolean,
  provider: ProviderProgress,
): SetupModel {
  const active = setup.organizations.filter((item) => item.state === "active");
  const suspended = setup.organizations
    .filter((item) => item.state === "suspended")
    .map((item) => item.login);
  const ready = machines.find(canStartVms);
  const blocked = ready === undefined ? machines[0] : undefined;
  const fix =
    blocked === undefined
      ? null
      : virtualizationProblem(blocked.virtualization);
  const job = firstJob(setup.first_job);
  const githubDone = active.length > 0;
  const machineDone = ready !== undefined && !holdMachine;
  const jobDone = job.kind === "finished";
  const providerStep = providerStatus(provider, machineDone);
  const jobOpen =
    machineDone && (providerStep !== "active" || job.kind === "moving");
  const steps: SetupStep[] = [
    {
      key: "github",
      title: "Install the GitHub App",
      status: githubDone ? "done" : "active",
      summary: githubDone
        ? `Installed on ${joinWords(
            active.map((item) => item.login),
            "and",
          )}`
        : `GitHub suspended the App on ${joinWords(suspended, "and")}.`,
    },
    {
      key: "machine",
      title: "Add a machine",
      status: machineDone ? "done" : "active",
      summary: ready
        ? machineSummary(ready)
        : "Run one command on a Linux machine or a Mac.",
    },
    {
      key: "provider",
      title: "Connect an overflow provider",
      status: providerStep,
      summary:
        provider.connected ??
        (provider.skipped
          ? "Skipped for now."
          : "Jobs start on a provider when your machines are busy."),
    },
    {
      key: "job",
      title: "Run the first job",
      status: jobDone ? "done" : jobOpen ? "active" : "todo",
      summary: jobSummary(job),
    },
  ];
  return {
    steps,
    done: steps.filter((step) => step.status === "done").length,
    total: steps.length,
    complete: githubDone && machineDone && jobDone,
    connected: ready === undefined ? null : connectedMachine(ready),
    cannotStartVms:
      blocked === undefined || fix === null
        ? null
        : { host: blocked.hostname, fix },
    suspended,
    job,
    imageWait: imageWait(setup, ready, machines, provider),
  };
}
