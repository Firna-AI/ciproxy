// The savings report of mokly-ai for the last 30 days, 19 October to 17
// November 2026, all on Team with three own machines. Source: the CI of
// mokly-ai (.context/ciproxy-pricing).

import { dateRange } from "./format.ts";
import { ax42 } from "./machines.ts";
import { tiers } from "./tiers.ts";

export const savingsPeriod = dateRange("19 October", "17 November");

const ownCost = 3 * ax42.monthlyCost;
const boatCost = 100;

export const savings = {
  starts: { own: 13_329, boat: 4_443 },
  minutes: { own: 81_250, boat: 34_819 },
  cost: { own: ownCost, boat: boatCost, plan: tiers.team.price },
  /** The same jobs at list prices, for the same VM sizes. */
  alternatives: { blacksmith: 698, github: 1_045 },
} as const;

export const yourCost =
  savings.cost.own + savings.cost.boat + savings.cost.plan;

/** Starts per day, own machines and Boat. The busiest day was 12 November. */
export const dailyStarts = {
  /** Day of the month; the chart caption names the months. */
  labels: [
    "19",
    "20",
    "21",
    "22",
    "23",
    "24",
    "25",
    "26",
    "27",
    "28",
    "29",
    "30",
    "31",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    "10",
    "11",
    "12",
    "13",
    "14",
    "15",
    "16",
    "17",
  ],
  own: [
    507, 512, 490, 539, 493, 132, 127, 489, 514, 491, 492, 514, 147, 102, 527,
    524, 523, 512, 501, 130, 113, 496, 494, 506, 1640, 539, 106, 132, 527, 510,
  ],
  boat: [
    138, 202, 128, 187, 165, 1, 6, 130, 172, 149, 189, 126, 18, 7, 199, 126,
    193, 125, 124, 4, 13, 187, 191, 189, 976, 205, 3, 18, 142, 130,
  ],
} as const;
