// The moments that the live queue shows, from the fixtures.

import type { Job } from "../../fixtures/jobs.ts";
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

export const liveStates = [
  "steady",
  "peak",
  "spend-limit",
  "starts-passed",
  "quiet",
  "loading",
  "error",
] as const;

export type LiveState = (typeof liveStates)[number];

export type LiveScene = {
  tier: TierKey;
  running: number;
  waiting: number;
  ownBusy: number;
  ownSlots: number;
  onBoat: number;
  warm: number;
  runningJobs: readonly Job[];
  waitingJobs: readonly Job[];
};

const steadySlots = machineScenes.steady;
const peakSlots = machineScenes.peak;
const quietSlots = machineScenes.quiet;

const steady: LiveScene = {
  tier: "team",
  running: 21,
  waiting: 0,
  ownBusy: countSlots(steadySlots, "busy"),
  ownSlots: 18,
  onBoat: 6,
  warm: countSlots(steadySlots, "warm"),
  runningJobs: steadyRunning,
  waitingJobs: [],
};

export const liveScenes: Readonly<Record<LiveState, LiveScene>> = {
  steady,
  peak: {
    tier: "team",
    running: 64,
    waiting: 12,
    ownBusy: countSlots(peakSlots, "busy"),
    ownSlots: 18,
    onBoat: 46,
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
    onBoat: 0,
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
    onBoat: 1,
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
    onBoat: 0,
    warm: countSlots(quietSlots, "warm"),
    runningJobs: [],
    waitingJobs: [],
  },
  loading: steady,
  error: steady,
};
