// The moments of the live queue whose wait reason comes from current facts of
// the API: an unknown fit with unknown own slots, the largest Boat size, Boat
// unable to take jobs, a full queue with a label that only Boat runs, a full
// queue while Boat costs are not counted, the Boat limit while a label that
// only Boat runs waits, a refused Boat size of one label, a label that no
// permitted place fits, a removed organization, and a quiet queue with no
// warm runners. Each list shows every job that the API sends, so no row says
// "N more". A full queue lists one running job in each own slot.

import { none, own, rows, type Job } from "../../fixtures/jobs.ts";
import {
  limitRunning,
  limitWaiting,
  steadyRunning,
} from "../../fixtures/live-jobs.ts";
import { machineScenes } from "../../fixtures/machines.ts";
import { boatOnlyLabel } from "../../fixtures/settings.ts";
import { nothingFits } from "../../fixtures/waiting-jobs.ts";
import { largestBoat, unknownFit } from "../../fixtures/wait-facts.ts";
import type { LiveScene } from "./live-scenes.ts";

export const factStates = [
  "unknown-fit",
  "largest-boat",
  "boat-key",
  "boat-billing",
  "overflow-off",
  "boat-not-connected",
  "boat-at-once",
  "spend-uncounted",
  "limit-boat-only",
  "size-refused",
  "nothing-fits-request",
  "boat-forbidden",
  "app-removed",
  "quiet-no-warm",
] as const;

/** Two jobs of a label that no own machine runs, while their sandboxes start. */
export const boatOnlyWaiting = rows("boat-only-wait", [
  [
    "render (sheets)",
    "Render",
    "mokly-cloud",
    "#352",
    "waiting",
    none,
    "6s",
    boatOnlyLabel.name,
  ],
  [
    "render (pdf)",
    "Render",
    "mokly-cloud",
    "#352",
    "waiting",
    none,
    "6s",
    boatOnlyLabel.name,
  ],
]);

export type FactState = (typeof factStates)[number];

/** The own slots of the three machines: six on each Hetzner AX42. */
const ownSlotCount = machineScenes.peak.reduce(
  (sum, machine) => sum + machine.slots.length,
  0,
);

/**
 * Thursday 26 November, 11:40: the other 14 jobs that run beside the four
 * of `limitRunning`, so that each of the 18 own slots runs one job.
 */
const otherSlotJobs = rows("full-run", [
  ["clippy", "CI", "mokly", "#1561", "running", own(1, 2), "5m 18s"],
  ["docs (build)", "Docs", "mokly", "#1561", "running", own(1, 3), "3m 40s"],
  [
    "bench (catalogue)",
    "Bench",
    "mokly",
    "main",
    "running",
    own(1, 4),
    "3m 12s",
  ],
  ["lint (css)", "CI", "mokly", "#1562", "running", own(1, 5), "1m 44s"],
  ["test (wasm)", "CI", "mokly", "#1562", "running", own(1, 6), "1m 40s"],
  [
    "build (linux-arm64)",
    "Release",
    "mokly-cloud",
    "#327",
    "running",
    own(2, 3),
    "2m 12s",
  ],
  [
    "integration (postgres)",
    "CI",
    "mokly-cloud",
    "#327",
    "running",
    own(2, 4),
    "2m 10s",
  ],
  [
    "integration (redis)",
    "CI",
    "mokly-cloud",
    "#327",
    "running",
    own(2, 5),
    "2m 09s",
  ],
  ["migrations", "CI", "mokly-cloud", "#327", "running", own(2, 6), "2m 02s"],
  [
    "unit (node 22)",
    "CI",
    "mokly-cloud",
    "#328",
    "running",
    own(3, 2),
    "1m 30s",
  ],
  [
    "eval (agents)",
    "Evals",
    "eval-runner",
    "main",
    "running",
    own(3, 3),
    "58s",
  ],
  ["eval (tools)", "Evals", "eval-runner", "main", "running", own(3, 4), "57s"],
  [
    "docker image",
    "Release",
    "mokly-cloud",
    "#328",
    "running",
    own(3, 5),
    "41s",
  ],
  ["storybook (a11y)", "CI", "mokly", "#1562", "running", own(3, 6), "35s"],
]);

/** Every own slot runs a job, and no job runs on Boat. */
const fullRunning: readonly Job[] = [...limitRunning, ...otherSlotJobs];

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
    ownSlots: ownSlotCount,
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
const full = scene(fullRunning, limitWaiting);

/** The full queue, with two jobs of a label that only Boat runs. */
const withBoatOnly = scene(fullRunning, [...limitWaiting, ...boatOnlyWaiting]);

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
  "boat-at-once": withBoatOnly,
  "spend-uncounted": full,
  "limit-boat-only": withBoatOnly,
  "size-refused": withBoatOnly,
  "nothing-fits-request": scene(steadyRunning, nothingFits.waiting),
  "boat-forbidden": scene(steadyRunning, nothingFits.waiting),
  "app-removed": scene(ownOrganizationJobs, [], { warm: 2 }),
  "quiet-no-warm": scene([], []),
};
