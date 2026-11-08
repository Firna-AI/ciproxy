// The four steps of Setup: the GitHub App, a machine, Boat, and the first
// job. Every fact comes from the API: the organizations, the machines that
// joined, the Boat account, and the first job with its phase and its result.
// A job counts as passed only when GitHub reports success. The Boat step can
// be skipped, and it never holds Setup open: Setup is complete when the
// first job finished, because the API keeps no record of a skip.

import type { MachineView, SetupJob, SetupView } from "../api/schema.ts";
import {
  countText,
  formatDuration,
  formatGigabytes,
  joinWords,
} from "./format.ts";

export type StepStatus = "done" | "active" | "todo" | "skipped";

export type SetupStep = {
  key: "github" | "machine" | "boat" | "job";
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

/** What Setup knows of Boat: the line of a connected account, or a skip. */
export type BoatProgress = {
  /** Set while a Boat account with an accepted key is connected. */
  connected: string | null;
  /** Skip for now in this tab, which counts only while Boat is not connected. */
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

function boatStatus(boat: BoatProgress, machineDone: boolean): StepStatus {
  if (boat.connected !== null) return "done";
  if (boat.skipped) return "skipped";
  return machineDone ? "active" : "todo";
}

/** Setup is complete when the first job finished, with or without Boat. */
export function setupComplete(
  setup: SetupView,
  machines: readonly MachineView[],
): boolean {
  return (
    setup.organizations.some((item) => item.state === "active") &&
    machines.some((machine) => machine.kvm) &&
    firstJob(setup.first_job).kind === "finished"
  );
}

/**
 * The model of Setup. `holdMachine` keeps the machine step open after a
 * machine joined while the page was open, until the person continues. The
 * first job opens after the Boat step, or at once when a job already runs.
 */
export function setupModel(
  setup: SetupView,
  machines: readonly MachineView[],
  holdMachine: boolean,
  boat: BoatProgress,
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
  const boatStep = boatStatus(boat, machineDone);
  const jobOpen =
    machineDone && (boatStep !== "active" || job.kind === "moving");
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
        ? `${ready.hostname} · ${countText(ready.slots, "slot", "slots")}`
        : "Run one command on a Linux machine.",
    },
    {
      key: "boat",
      title: "Connect Boat for overflow",
      status: boatStep,
      summary:
        boat.connected ??
        (boat.skipped
          ? "Skipped for now."
          : "Jobs start on Boat when your machines are busy."),
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
    noKvm,
    suspended,
    job,
  };
}
