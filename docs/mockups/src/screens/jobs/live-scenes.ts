// The moments that the live queue shows, from the fixtures. Each moment
// counts the running jobs of each connected provider; in the main story,
// Boat is the only one.

import { providerOf, type Job } from "../../fixtures/jobs.ts";
import {
  freeRunning,
  limitRunning,
  limitWaiting,
  peakRunning,
  peakWaiting,
  steadyRunning,
} from "../../fixtures/live-jobs.ts";
import { countSlots, machineScenes } from "../../fixtures/machines.ts";
import type { TierKey } from "../../fixtures/org.ts";
import type { ProviderKey } from "../../fixtures/providers.ts";
import { boatRefused, nothingFits } from "../../fixtures/waiting-jobs.ts";
import { factScenes, factStates } from "./live-fact-scenes.ts";

export const liveStates = [
  "steady",
  "peak",
  "spend-limit",
  "starts-passed",
  "quiet",
  "boat-refused",
  "nothing-fits",
  "app-suspended",
  "loading",
  "error",
  ...factStates,
  "suspended-current",
  "refused-current",
] as const;

export type LiveState = (typeof liveStates)[number];

/** The jobs that run on one provider now. */
export type ProviderCount = { provider: ProviderKey; running: number };

/** The running jobs on Boat, the one provider of the main story. */
export function onBoat(running: number): readonly ProviderCount[] {
  return [{ provider: "boat", running }];
}

/** The running jobs of a scene on one provider, 0 when it runs none. */
export function runningOn(scene: LiveScene, provider: ProviderKey): number {
  const entry = scene.onProviders.find((count) => count.provider === provider);
  return entry?.running ?? 0;
}

/** The running jobs of a list on each provider, for the moments that list all. */
export function countOn(
  jobs: readonly Job[],
  provider: ProviderKey,
): ProviderCount {
  const running = jobs.filter((job) => providerOf(job.where) === provider);
  return { provider, running: running.length };
}

export type LiveScene = {
  tier: TierKey;
  running: number;
  waiting: number;
  ownBusy: number;
  ownSlots: number;
  /** The running jobs of each connected provider, in the order of connection. */
  onProviders: readonly ProviderCount[];
  warm: number;
  runningJobs: readonly Job[];
  waitingJobs: readonly Job[];
  /** ciproxy cannot see the jobs that wait, as while the App is suspended. */
  waitingUnknown?: true;
  /** ciproxy does not know the free own slots, as while a report is missing. */
  slotsUnknown?: true;
};

const steadySlots = machineScenes.steady;
const peakSlots = machineScenes.peak;
const quietSlots = machineScenes.quiet;

/** Thursday 19 November: the jobs that started before the suspension. */
const suspendedRunning = steadyRunning.slice(0, 7);

/** The slots of the three own machines. */
const steadySlotCount = steadySlots.reduce(
  (sum, machine) => sum + machine.slots.length,
  0,
);

function countWhere(jobs: readonly Job[], kind: Job["where"]["kind"]) {
  return jobs.filter((job) => job.where.kind === kind).length;
}

const steady: LiveScene = {
  tier: "team",
  running: 21,
  waiting: 0,
  ownBusy: countSlots(steadySlots, "busy"),
  ownSlots: 18,
  onProviders: onBoat(6),
  warm: countSlots(steadySlots, "warm"),
  runningJobs: steadyRunning,
  waitingJobs: [],
};

const appSuspended: LiveScene = {
  tier: "team",
  running: suspendedRunning.length,
  waiting: 0,
  ownBusy: countWhere(suspendedRunning, "own"),
  ownSlots: steadySlotCount,
  onProviders: [countOn(suspendedRunning, "boat")],
  warm: 0,
  runningJobs: suspendedRunning,
  waitingJobs: [],
  waitingUnknown: true,
};

const boatRefusedScene: LiveScene = {
  tier: "team",
  running: 54,
  waiting: 9,
  ownBusy: 18,
  ownSlots: 18,
  onProviders: onBoat(36),
  warm: 0,
  runningJobs: boatRefused.running,
  waitingJobs: boatRefused.waiting,
};

export const liveScenes: Readonly<Record<LiveState, LiveScene>> = {
  steady,
  peak: {
    tier: "team",
    running: 64,
    waiting: 12,
    ownBusy: countSlots(peakSlots, "busy"),
    ownSlots: 18,
    onProviders: onBoat(46),
    warm: 0,
    runningJobs: peakRunning,
    waitingJobs: peakWaiting,
  },
  "spend-limit": {
    tier: "team",
    running: 18,
    waiting: 23,
    ownBusy: 18,
    ownSlots: 18,
    onProviders: onBoat(0),
    warm: 0,
    runningJobs: limitRunning,
    waitingJobs: limitWaiting,
  },
  "starts-passed": {
    tier: "free",
    running: 6,
    waiting: 0,
    ownBusy: 5,
    ownSlots: 6,
    onProviders: onBoat(1),
    warm: 1,
    runningJobs: freeRunning,
    waitingJobs: [],
  },
  quiet: {
    tier: "team",
    running: 0,
    waiting: 0,
    ownBusy: 0,
    ownSlots: 18,
    onProviders: onBoat(0),
    warm: countSlots(quietSlots, "warm"),
    runningJobs: [],
    waitingJobs: [],
  },
  "boat-refused": boatRefusedScene,
  "nothing-fits": {
    ...steady,
    running: 19,
    waiting: 2,
    onProviders: onBoat(4),
    waitingJobs: nothingFits.waiting,
  },
  "app-suspended": appSuspended,
  loading: steady,
  error: steady,
  ...factScenes,
  "suspended-current": appSuspended,
  "refused-current": boatRefusedScene,
};
