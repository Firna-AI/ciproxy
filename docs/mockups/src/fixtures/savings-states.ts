// The figures of each Savings state. The known report is the last 30 days of
// the release, from savings.ts. The other states change only what their case
// needs: a cost that is not set or known in part, an alternative without a
// complete price, a quiet day on which ciproxy costs more than Blacksmith, a
// report that still refreshes, and empty ranges. The current-API report
// names no tier and no machine count, because the API gives neither.

import { dateRange, formatDay } from "./format.ts";
import { ax42 } from "./hardware.ts";
import { dailyStarts, savings, savingsPeriod } from "./savings.ts";
import { ownMachines, tiers } from "./tiers.ts";

export type Alternative = "blacksmith" | "github";

export type ReportRange = "30" | "month" | "last";

export type ReportFigures = {
  range: ReportRange;
  /** The first and the last included UTC day, such as "1 November". */
  period: string;
  starts: { own: number; boat: number };
  minutes: { own: number; boat: number };
  /** The known cost of the own machines, or null when none is set. */
  ownCost: number | null;
  /** Some machine costs are missing, so ownCost is the known part. */
  ownPartial?: true;
  boatCost: number;
  planCost: number;
  /** Some plan prices are not known, so planCost is the known part. */
  planPartial?: true;
  alternatives: Readonly<Record<Alternative, number | null>>;
  /** An alternative without a complete price, and the size it lacks. */
  priceGap?: { alternative: Alternative; size: string };
  ownLabel: string;
  planLabel: string;
  daily: {
    labels: readonly string[];
    own: readonly number[];
    boat: readonly number[];
  };
  /** The report still refreshes, so the page reads it again. */
  updating?: true;
};

/** The complete cost with ciproxy, or null while a cost is missing. */
export function totalCost(figures: ReportFigures): number | null {
  if (figures.ownCost === null || figures.ownPartial || figures.planPartial) {
    return null;
  }
  return figures.ownCost + figures.boatCost + figures.planCost;
}

/** The cheaper alternative, when every cost is known. */
export function leadOf(figures: ReportFigures): Alternative | null {
  const { blacksmith, github } = figures.alternatives;
  if (totalCost(figures) === null || blacksmith === null || github === null) {
    return null;
  }
  return blacksmith <= github ? "blacksmith" : "github";
}

const known: ReportFigures = {
  range: "30",
  period: savingsPeriod,
  starts: savings.starts,
  minutes: savings.minutes,
  ownCost: savings.cost.own,
  boatCost: savings.cost.boat,
  planCost: savings.cost.plan,
  alternatives: savings.alternatives,
  ownLabel: ownMachines(3),
  planLabel: `ciproxy ${tiers.team.name}`,
  daily: dailyStarts,
};

const none = { own: 0, boat: 0 } as const;

const emptyFigures: ReportFigures = {
  ...known,
  starts: none,
  minutes: none,
  boatCost: 0,
  alternatives: { blacksmith: 0, github: 0 },
  daily: { labels: [], own: [], boat: [] },
};

/** Sunday 1 November: the first day of the month, quiet as a weekend. */
const quietDay = formatDay(new Date(Date.UTC(2026, 10, 1)));

export const savingsFigures = {
  report: known,
  "missing-costs": { ...known, ownCost: null },
  negative: {
    ...known,
    range: "month",
    period: quietDay,
    starts: { own: 102, boat: 7 },
    minutes: { own: 498, boat: 41 },
    ownCost: (3 * ax42.monthlyCost) / 30,
    boatCost: 0.16,
    planCost: tiers.team.price / 30,
    alternatives: { blacksmith: 5.86, github: 8.62 },
    daily: { labels: ["1"], own: [102], boat: [7] },
  },
  "missing-price": {
    ...known,
    alternatives: { blacksmith: null, github: savings.alternatives.github },
    priceGap: { alternative: "blacksmith", size: "16 vCPU and 64 GB" },
  },
  "partial-costs": {
    ...known,
    ownCost: (savings.cost.own * 2) / 3,
    ownPartial: true,
  },
  "unknown-costs": {
    ...known,
    planCost: (savings.cost.plan * 20) / 30,
    planPartial: true,
  },
  updating: { ...known, updating: true },
  empty: emptyFigures,
  "empty-range": {
    ...emptyFigures,
    range: "last",
    period: dateRange(
      formatDay(new Date(Date.UTC(2026, 9, 1))),
      formatDay(new Date(Date.UTC(2026, 9, 31))),
    ),
  },
  current: { ...known, ownLabel: "Own machines", planLabel: "ciproxy plan" },
  loading: known,
  error: known,
} as const satisfies Readonly<Record<string, ReportFigures>>;

export type SavingsState = keyof typeof savingsFigures;
