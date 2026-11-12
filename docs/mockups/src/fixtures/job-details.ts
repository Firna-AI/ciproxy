// The finished jobs that the job page shows, on Wednesday 18 November: one
// that a warm runner took on an own machine, one that ran on Boat, one whose
// machine cache was not available, so it used the GitHub cache, one of
// mokly-labs that restored from both caches, and one that restored no entry.
// Only a job whose restores the machine cache handled shows the cache counts.
//
// The size of a job is the request that its label gave it at the start, which
// ciproxy saves. In a what-if, ciproxy has no saved request for two jobs of
// the history, one on an own machine and one on Boat. Their request is then
// unknown, never the current size of the label, which can change after a job.
// The VM that ran each of them stays known.

import { factDetails } from "./job-details-facts.ts";
import {
  onBoat,
  own,
  repositoryName,
  type JobState,
  type Where,
} from "./jobs.ts";
import { boat } from "./machines.ts";
import { labelNamed } from "./settings.ts";
import {
  boatSizeFor,
  boatSizeNamed,
  type BoatSize,
  type JobVm,
  type Size,
} from "./sizes.ts";
import { vmDiskBytes } from "./vm-disk.ts";

/** The counts of the restores that the machine cache handled. */
type CacheCounts = { restored: number; sizeGb: number };

/**
 * Where the restores of a job came from, and their counts when known. The
 * machine cache counts the restores that it handled, also those whose
 * contents came from the GitHub cache: "both" names the two caches.
 */
export type CacheUse =
  | ({ source: "machine" } & CacheCounts)
  | ({ source: "both" } & CacheCounts)
  | { source: "github"; fallback: boolean }
  | { source: "none" }
  /** The API gives neither a cache source nor counts. */
  | { source: "unknown" };

/** One line of the timeline; a machine name in it shows in monospace. */
export type JobEvent = { time: string; text: string; machine?: string };

export type JobDetail = {
  name: string;
  context: string;
  /** The name and the ref of the job in the history list. */
  ref: string;
  where: Where;
  /** The name of the label in runs-on. */
  label: string;
  /** The size that the label gave the job at its start, when ciproxy has it. */
  request: Size | "unknown";
  /** The Boat size that ran a job on Boat. */
  boatSize?: BoatSize;
  /** The VM that ran a job on an own machine, shown for an unknown request. */
  vm?: JobVm;
  /** Why the job ran there, or null when the API gives no reason. */
  why: string | null;
  /** The result in the header; Passed when it is not given. */
  result?: JobState;
  /** No ciproxy runner took the job, so it is not a start. */
  noRunner?: true;
  /** A page of the current API, which gives no fit for a Boat size. */
  currentContract?: true;
  timeline: readonly JobEvent[];
  waited: string;
  duration: string;
  cache: CacheUse;
};

/** Every job of the page uses the default label. */
const label = labelNamed("ciproxy");

export const jobDetailStates = [
  "own",
  "boat",
  "fallback",
  "mixed-cache",
  "no-restores",
  "unknown-request",
  "unknown-request-boat",
  "no-runner",
  "unknown-facts",
  "current-own",
  "current-boat",
] as const;
export type JobDetailState = (typeof jobDetailStates)[number];

const warmReason = "A warm runner was ready, so the job started at once.";

const boatReason = `Your machines were full, so the job started on Boat after ${boat.waitSeconds} seconds.`;

export const jobDetails: Readonly<Record<JobDetailState, JobDetail>> = {
  own: {
    name: "test (rust)",
    context: `${repositoryName("mokly")} · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: own(1, 3),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:25:09", text: "Queued on GitHub" },
      {
        time: "14:25:09",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-1",
      },
      { time: "14:31:21", text: "Passed" },
    ],
    waited: "0s",
    duration: "6m 12s",
    cache: { source: "machine", restored: 3, sizeGb: 2.1 },
  },
  boat: {
    name: "e2e (chromium) 3/4",
    context: `${repositoryName("mokly")} · E2E · main`,
    ref: "main",
    where: onBoat,
    label: label.name,
    request: label,
    boatSize: boatSizeFor(label),
    why: boatReason,
    timeline: [
      { time: "14:23:04", text: "Queued on GitHub" },
      { time: "14:23:04", text: "No free slot on your machines" },
      { time: "14:23:34", text: "Runner started on Boat" },
      { time: "14:23:35", text: "Taken by the Boat runner" },
      { time: "14:30:40", text: "Passed" },
    ],
    waited: "31s",
    duration: "7m 05s",
    cache: { source: "github", fallback: false },
  },
  fallback: {
    name: "clippy",
    context: `${repositoryName("mokly")} · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: own(2, 4),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:26:20", text: "Queued on GitHub" },
      {
        time: "14:26:20",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-2",
      },
      { time: "14:29:08", text: "Passed" },
    ],
    waited: "0s",
    duration: "2m 48s",
    cache: { source: "github", fallback: true },
  },
  "mixed-cache": {
    name: "eval (agents)",
    context: `${repositoryName("eval-runner")} · Evals · main`,
    ref: "main",
    where: own(2, 3),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:20:00", text: "Queued on GitHub" },
      {
        time: "14:20:00",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-2",
      },
      { time: "14:25:08", text: "Passed" },
    ],
    waited: "0s",
    duration: "5m 08s",
    cache: { source: "both", restored: 4, sizeGb: 3.6 },
  },
  "no-restores": {
    name: "fmt",
    context: `${repositoryName("mokly")} · CI · #1538 Restyle the catalogue sidebar`,
    ref: "#1538",
    where: own(2, 6),
    label: label.name,
    request: label,
    why: warmReason,
    timeline: [
      { time: "14:18:41", text: "Queued on GitHub" },
      {
        time: "14:18:41",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-2",
      },
      { time: "14:19:02", text: "Passed" },
    ],
    waited: "0s",
    duration: "21s",
    cache: { source: "none" },
  },
  "unknown-request": {
    name: "storybook",
    context: `${repositoryName("mokly")} · CI · #1537 Upgrade Storybook`,
    ref: "#1537",
    where: own(1, 2),
    label: label.name,
    request: "unknown",
    vm: { vcpu: 2, memoryGb: 8, vmDiskBytes: vmDiskBytes({ freeDiskGb: 20 }) },
    why: warmReason,
    timeline: [
      { time: "14:13:27", text: "Queued on GitHub" },
      {
        time: "14:13:27",
        text: "Taken by a warm runner on",
        machine: "hetzner-ax42-1",
      },
      { time: "14:17:00", text: "Passed" },
    ],
    waited: "0s",
    duration: "3m 33s",
    cache: { source: "machine", restored: 2, sizeGb: 0.9 },
  },
  "unknown-request-boat": {
    name: "docker image",
    context: `${repositoryName("mokly-cloud")} · Release · main`,
    ref: "main",
    where: onBoat,
    label: label.name,
    request: "unknown",
    boatSize: boatSizeNamed("large"),
    why: boatReason,
    timeline: [
      { time: "14:19:08", text: "Queued on GitHub" },
      { time: "14:19:08", text: "No free slot on your machines" },
      { time: "14:19:38", text: "Runner started on Boat" },
      { time: "14:19:40", text: "Taken by the Boat runner" },
      { time: "14:24:20", text: "Passed" },
    ],
    waited: "32s",
    duration: "4m 40s",
    cache: { source: "github", fallback: false },
  },
  ...factDetails,
};
