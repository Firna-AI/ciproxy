// The words and formats of a job: its result, a duration, the request of its
// label, the VM that ran it, where it ran, and its context. Each value comes
// from the API. An absent value gives null or "Unknown", never zero, and an
// executed job never borrows the current size of its label.

import type { StatusTone } from "@ciproxy/tokens";

import type {
  JobLabel,
  JobLocation,
  JobView,
  JobVmSize,
} from "../api/jobs-types.ts";
import { dotJoin, unbroken } from "./exact-format.ts";
import { formatGigabytes } from "./format.ts";

export const unknownText = "Unknown";

export type ResultLook = { label: string; tone: StatusTone };

/** The results of GitHub and of the scale sets, which the API keeps verbatim. */
const results: Readonly<Record<string, ResultLook>> = {
  success: { label: "Passed", tone: "ok" },
  succeeded: { label: "Passed", tone: "ok" },
  failure: { label: "Failed", tone: "err" },
  failed: { label: "Failed", tone: "err" },
  cancelled: { label: "Cancelled", tone: "idle" },
  canceled: { label: "Cancelled", tone: "idle" },
  timed_out: { label: "Timed out", tone: "err" },
  startup_failure: { label: "Startup failure", tone: "err" },
  action_required: { label: "Action required", tone: "warn" },
  skipped: { label: "Skipped", tone: "idle" },
  neutral: { label: "Neutral", tone: "idle" },
  stale: { label: "Stale", tone: "idle" },
};

/** The result of a finished job as a word and a status tone. */
export function resultLook(result: string | null | undefined): ResultLook {
  if (result === null || result === undefined) {
    return { label: "Finished", tone: "idle" };
  }
  if (Object.hasOwn(results, result)) {
    return results[result] ?? { label: result, tone: "idle" };
  }
  const words = result.replaceAll("_", " ");
  return {
    label: words.charAt(0).toUpperCase() + words.slice(1),
    tone: "idle",
  };
}

/** The state of any job: running, waiting, or its result. */
export function jobLook(job: JobView): ResultLook {
  if (job.status === "running") return { label: "Running", tone: "run" };
  if (job.status === "queued") return { label: "Waiting", tone: "warn" };
  return resultLook(job.result);
}

/** Whole seconds of an exact decimal, half up: "372.5" -> 373. */
function wholeSeconds(value: string): number {
  const [whole = "0", fraction = ""] = value.split(".");
  return Number(whole) + ((fraction[0] ?? "0") >= "5" ? 1 : 0);
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

/** "6m 12s", "45s", or "1h 04m": exact seconds as a duration. */
export function secondsText(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const rest = seconds % 60;
  if (hours > 0) return `${hours}h ${pad(minutes)}m`;
  if (minutes > 0) return `${minutes}m ${pad(rest)}s`;
  return `${rest}s`;
}

/** A decimal duration of the API, or "Unknown" when it is absent. */
export function durationText(seconds: string | null | undefined): string {
  if (seconds === null || seconds === undefined) return unknownText;
  return secondsText(wholeSeconds(seconds));
}

/**
 * How long a waiting job has waited at the time of the read. The API gives
 * the wait only after a runner takes the job, so this counts from the queue
 * time of GitHub. A missing time or a clock behind it gives null.
 */
export function waitingText(
  queuedAt: string | null | undefined,
  now: number,
): string | null {
  if (queuedAt === null || queuedAt === undefined) return null;
  const elapsed = Math.floor((now - Date.parse(queuedAt)) / 1000);
  return Number.isFinite(elapsed) && elapsed >= 0 ? secondsText(elapsed) : null;
}

/**
 * "2 vCPU · 8 GB · 20 GB free disk": the request of the label, or null when
 * the API did not keep it. One missing value makes the request unknown.
 */
export function requestText(label: JobLabel): string | null {
  const { vcpus, memory_gb: memory, disk_gb: disk } = label;
  if (vcpus === null || memory === null || disk === null) return null;
  return dotJoin(
    unbroken(`${vcpus} vCPU`),
    unbroken(`${memory} GB`),
    unbroken(`${disk} GB free disk`),
  );
}

/** "2 vCPU · 8 GB · 81 GB VM disk": the VM that ran a job on a machine. */
export function vmText(size: JobVmSize): string {
  return dotJoin(
    unbroken(`${size.vcpus} vCPU`),
    unbroken(formatGigabytes(size.memory_bytes)),
    unbroken(`${formatGigabytes(size.disk_bytes)} VM disk`),
  );
}

/** "Boat large · 8 vCPU · 16 GB · 111 GB disk": the Boat size that ran it. */
export function boatText(name: string, size: JobVmSize | null): string {
  if (size === null) return `Boat ${name}`;
  return dotJoin(
    unbroken(`Boat ${name}`),
    unbroken(`${size.vcpus} vCPU`),
    unbroken(formatGigabytes(size.memory_bytes)),
    unbroken(`${formatGigabytes(size.disk_bytes)} disk`),
  );
}

export type Place =
  | { kind: "machine"; name: string; slot: string | null }
  | { kind: "boat" }
  | { kind: "none" }
  | { kind: "unknown" };

/**
 * Where a job runs or ran. A job with no place that no runner took had no
 * runner. A slot shows from 1, as the Machines page counts them.
 */
export function placeOf(
  location: JobLocation | null | undefined,
  countsAsStart: boolean,
): Place {
  if (location === null || location === undefined) {
    return countsAsStart ? { kind: "unknown" } : { kind: "none" };
  }
  if (location.kind === "boat") return { kind: "boat" };
  const slot = location.first_slot;
  return {
    kind: "machine",
    name: location.name ?? "Own machine",
    slot: slot === null || slot === undefined ? null : `slot ${slot + 1}`,
  };
}

/** The branch or tag of a job, else its full ref, else nothing. */
function refOf(job: JobView): string | null {
  return job.head_branch ?? job.git_ref ?? null;
}

/** "CI · mokly-ai/mokly main": the line under a job name in a list. */
export function contextText(job: JobView): string {
  const ref = refOf(job);
  return dotJoin(
    job.workflow,
    ref ? `${job.repository} ${ref}` : job.repository,
  );
}

/** "mokly-ai/mokly · CI · main": the subtitle of the job page. */
export function subtitleText(job: JobView): string {
  const ref = refOf(job);
  return ref
    ? dotJoin(job.repository, job.workflow, ref)
    : dotJoin(job.repository, job.workflow);
}
