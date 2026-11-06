// The three steps of Setup without its Boat step: the GitHub App, a machine,
// and the first job. Every fact comes from the API: the organizations, the
// machines that joined, and the first job with its phase and its result. A
// job counts as passed only when GitHub reports success.

import type { MachineView, SetupJob, SetupView } from "../api/schema.ts";
import {
  countText,
  formatDuration,
  formatGigabytes,
  joinWords,
} from "./format.ts";

export type StepStatus = "done" | "active" | "todo";

export type SetupStep = {
  key: "github" | "machine" | "job";
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
  /** The name of a machine that joined without KVM. */
  noKvm: string | null;
  /** Organizations whose App GitHub suspended. */
  suspended: readonly string[];
  job: FirstJob;
};

const outcomes: Readonly<Record<string, string>> = {
  failure: "failed",
  cancelled: "was cancelled",
  timed_out: "timed out",
  skipped: "was skipped",
};

function placeText(job: SetupJob): string {
  if (job.place === "machine") return " on your machine";
  if (job.place === "boat") return " on Boat";
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
      countText(machine.threads, "thread", "threads"),
      formatGigabytes(machine.memory_bytes),
      "KVM ready",
      countText(machine.slots, "slot", "slots"),
    ].join(" · "),
    diskNote:
      atOnce !== null && atOnce < machine.slots
        ? `Its disk holds ${countText(atOnce, "job", "jobs")} of ciproxy at a time.`
        : null,
  };
}

function jobSummary(job: FirstJob): string {
  if (job.kind !== "finished") return "Change runs-on in one workflow.";
  return job.passed ? "Your first job passed." : "Your first job finished.";
}

/**
 * The model of Setup. `holdMachine` keeps the machine step open after a
 * machine joined while the page was open, until the person continues.
 */
export function setupModel(
  setup: SetupView,
  machines: readonly MachineView[],
  holdMachine: boolean,
): SetupModel {
  const active = setup.organizations.filter((item) => item.state === "active");
  const suspended = setup.organizations
    .filter((item) => item.state === "suspended")
    .map((item) => item.login);
  const ready = machines.find((machine) => machine.kvm);
  const noKvm = ready === undefined ? (machines[0]?.hostname ?? null) : null;
  const job = firstJob(setup.first_job);
  const githubDone = active.length > 0;
  const machineDone = ready !== undefined && !holdMachine;
  const jobDone = job.kind === "finished";
  const machineStatus: StepStatus = machineDone ? "done" : "active";
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
      status: machineStatus,
      summary: ready
        ? `${ready.hostname} · ${countText(ready.slots, "slot", "slots")}`
        : "Run one command on a Linux machine.",
    },
    {
      key: "job",
      title: "Run the first job",
      status: jobDone ? "done" : machineDone ? "active" : "todo",
      summary: jobSummary(job),
    },
  ];
  const done = steps.filter((step) => step.status === "done").length;
  return {
    steps,
    done,
    total: steps.length,
    complete: done === steps.length,
    connected: ready === undefined ? null : connectedMachine(ready),
    noKvm,
    suspended,
    job,
  };
}
