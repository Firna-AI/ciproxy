// The savings report of mokly-ai for the last 30 days on the day of the
// release of the overflow flow, Wednesday 18 November 2026: the 30 UTC days
// from 20 October to that day, all on Team with three own machines. Both the
// range and its labels come from that day. The days up to 17 November come
// from the CI of mokly-ai (.context/ciproxy-pricing). The release day counts
// the starts up to 14:32: the 612 finished jobs and the 21 that run, with the
// overflow of the release on Boat. The minutes and the costs count the same
// days.
//
// A monthly price counts for each UTC day of the range: the price divided by
// the days of the calendar month of that day, so the range counts 12/31 of
// the October price and 18/30 of the November price. When a price changes on
// a day, the last price of that day counts for the whole day. No price
// changes in this range, and the three machines are registered on every day.

import { dateRange, formatDay } from "./format.ts";
import { ax42 } from "./hardware.ts";
import { releaseDay } from "./live-jobs.ts";
import { tiers } from "./tiers.ts";

const dayMs = 86_400_000;

/** The report covers 30 UTC days, the day it shows included. */
const reportDayCount = 30;

/** The first and the last UTC day of the report. */
const lastDay = releaseDay;
const firstDay = new Date(lastDay.getTime() - (reportDayCount - 1) * dayMs);

export const savingsPeriod = dateRange(formatDay(firstDay), formatDay(lastDay));

/** Each UTC day of the report, the first and the last day included. */
const reportDays: readonly Date[] = Array.from(
  { length: reportDayCount },
  (_, index) => new Date(firstDay.getTime() + index * dayMs),
);

/** The days of the calendar month of a UTC day: 31 for 20 October. */
function monthDays(day: Date): number {
  const nextMonth = Date.UTC(day.getUTCFullYear(), day.getUTCMonth() + 1);
  return new Date(nextMonth - dayMs).getUTCDate();
}

/** What a monthly price costs over the days of the report. */
function rangeCost(monthlyPrice: number): number {
  return reportDays.reduce(
    (sum, day) => sum + monthlyPrice / monthDays(day),
    0,
  );
}

/**
 * Starts per day, own machines and Boat, one value for each day of the
 * report. The busiest day was 12 November.
 */
export const dailyStarts = {
  /** Day of the month; the chart caption names the months. */
  labels: reportDays.map((day) => String(day.getUTCDate())),
  own: [
    512, 490, 539, 493, 132, 127, 489, 514, 491, 492, 514, 147, 102, 527, 524,
    523, 512, 501, 130, 113, 496, 494, 506, 1640, 539, 106, 132, 527, 510, 401,
  ],
  boat: [
    202, 128, 187, 165, 1, 6, 130, 172, 149, 189, 126, 18, 7, 199, 126, 193,
    125, 124, 4, 13, 187, 191, 189, 976, 205, 3, 18, 142, 130, 232,
  ],
} as const;

function total(values: readonly number[]): number {
  return values.reduce((sum, value) => sum + value, 0);
}

const ownCost = 3 * rangeCost(ax42.monthlyCost);
const boatCost = 102;

export const savings = {
  starts: { own: total(dailyStarts.own), boat: total(dailyStarts.boat) },
  minutes: { own: 80_604, boat: 35_556 },
  cost: { own: ownCost, boat: boatCost, plan: rangeCost(tiers.team.price) },
  /** The same jobs at list prices, for the same VM sizes. */
  alternatives: { blacksmith: 699, github: 1_046 },
} as const;

export const yourCost =
  savings.cost.own + savings.cost.boat + savings.cost.plan;
