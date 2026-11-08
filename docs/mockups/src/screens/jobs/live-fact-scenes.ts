// The moments of the live queue whose wait reason comes from current facts of
// the API: an unknown fit with unknown own slots, the largest Boat size, Boat
// unable to take jobs, a removed organization, and a quiet queue with no warm
// runners. Each list shows every job that the API sends, so no row says
// "N more".

import type { Job } from "../../fixtures/jobs.ts";
import {
  limitRunning,
  limitWaiting,
  steadyRunning,
} from "../../fixtures/live-jobs.ts";
import { largestBoat, unknownFit } from "../../fixtures/wait-facts.ts";
import type { LiveScene } from "./live-scenes.ts";

export const factStates = [
  "unknown-fit",
  "largest-boat",
  "boat-key",
  "boat-billing",
  "overflow-off",
  "boat-not-connected",
  "app-removed",
  "quiet-no-warm",
] as const;

export type FactState = (typeof factStates)[number];

function scene(
  runningJobs: readonly Job[],
  waitingJobs: readonly Job[],
  extra: Partial<LiveScene> = {},
): LiveScene {
  const own = runningJobs.filter((job) => job.where.kind === "own").length;
  return {
    tier: "team",
    running: runningJobs.length,
    waiting: waitingJobs.length,
    ownBusy: own,
    ownSlots: 18,
    onBoat: runningJobs.length - own,
    warm: 0,
    runningJobs,
    waitingJobs,
    ...extra,
  };
}

/** The jobs of mokly-ai only, after mokly-labs removed the App. */
const ownOrganizationJobs = steadyRunning.filter(
  (job) => job.repo !== "eval-runner",
);

/** A full queue: every own slot runs a job, and six jobs wait. */
const full = scene(limitRunning, limitWaiting);

export const factScenes: Readonly<Record<FactState, LiveScene>> = {
  "unknown-fit": scene(steadyRunning.slice(0, 8), unknownFit.waiting, {
    slotsUnknown: true,
  }),
  "largest-boat": scene(steadyRunning.slice(0, 6), largestBoat.waiting, {
    warm: 4,
  }),
  "boat-key": full,
  "boat-billing": full,
  "overflow-off": full,
  "boat-not-connected": full,
  "app-removed": scene(ownOrganizationJobs, [], { warm: 2 }),
  "quiet-no-warm": scene([], []),
};
