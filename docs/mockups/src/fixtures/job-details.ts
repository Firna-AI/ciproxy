// The finished jobs that the job page shows, on Wednesday 18 November: one
// that a warm runner took on an own machine, one that ran on Boat, and one
// whose machine cache was not available, so it used the GitHub cache. Only a
// job whose restores the machine cache handled shows the cache counts.

import { onBoat, own, type Where } from "./jobs.ts";
import { boat } from "./machines.ts";
import { org } from "./org.ts";
import { labelNamed, type Label } from "./settings.ts";
import { boatSizeFor, type BoatSize } from "./sizes.ts";

/** Where the restores of a job came from, and their counts when known. */
export type CacheUse =
  | { source: "machine"; restored: number; sizeGb: number }
  | { source: "github"; fallback: boolean };

/** One line of the timeline; a machine name in it shows in monospace. */
export type JobEvent = { time: string; text: string; machine?: string };

export type JobDetail = {
  name: string;
  context: string;
  /** The name and the ref of the job in the history list. */
  ref: string;
  where: Where;
  label: Label;
  /** The Boat size that ran a job on Boat. */
  boatSize?: BoatSize;
  why: string;
  timeline: readonly JobEvent[];
  waited: string;
  duration: string;
  cache: CacheUse;
};

/** Every job of the page uses the default label. */
const label = labelNamed("ciproxy");

export const jobDetailStates = ["own", "boat", "fallback"] as const;
export type JobDetailState = (typeof jobDetailStates)[number];

const warmReason = "A warm runner was ready, so the job started at once.";

export const jobDetails: Readonly<Record<JobDetailState, JobDetail>> = {
  own: {
    name: "test (rust)",
    context: `${org.login}/mokly · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: own(1, 3),
    label,
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
    context: `${org.login}/mokly · E2E · main`,
    ref: "main",
    where: onBoat,
    label,
    boatSize: boatSizeFor(label),
    why: `Your machines were full, so the job started on Boat after ${boat.waitSeconds} seconds.`,
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
    context: `${org.login}/mokly · CI · #1542 Add catalogue search`,
    ref: "#1542",
    where: own(2, 4),
    label,
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
};
