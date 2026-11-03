// Recent values behind the headline figures, at 14:32 on Wednesday 18
// November: one value every five minutes for the last hour, and the cache hit
// rate of each of the last 7 days. Only the steady scenes have trends. Each
// series ends at the figure the screen shows.

/** Busy own slots, of 18. Two bursts filled every slot, near 13:50 and 14:15. */
const busySlots = [12, 14, 18, 18, 16, 13, 15, 18, 18, 17, 16, 15];

/** Jobs on Boat. They start only while every own slot is busy. */
const onBoat = [0, 0, 3, 5, 4, 2, 0, 4, 8, 7, 7, 6];

/** Warm runners, up to 2 on each machine, in the slots that are free. */
const warmRunners = [6, 4, 0, 0, 2, 5, 3, 0, 0, 1, 2, 3];

/** Jobs that waited for a runner when the bursts began. */
const waiting = [0, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0];

/** The cache hit rate of each day, in percent; 91% over the 7 days. */
const cacheHits = [89, 92, 90, 93, 91, 90, 92];

export const machineTrends = {
  steady: { busySlots, warmRunners, cacheHits },
} as const;

export const liveTrends = {
  steady: {
    running: busySlots.map((busy, i) => busy + (onBoat[i] ?? 0)),
    waiting,
    busySlots,
    onBoat,
  },
} as const;
