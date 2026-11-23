// The figures of each Savings state. The known report is the last 30 days of
// the release, from savings.ts. The other states change only what their case
// needs: a cost that is not set or known in part, an alternative without a
// complete price, a quiet day on which ciproxy costs more than Blacksmith, a
// report that still refreshes, and empty ranges. The current-API report
// names no tier and no machine count, because the API gives neither. In the
// what-if with two providers, Second provider ran the overflow of
// ciproxy-large from 9 November, so a part of the Boat row moves to its own
// row, and the totals stay.

import { dateRange, formatDay } from "./format.ts";
import { ax42 } from "./hardware.ts";
import {
  dailyStarts,
  providerCost,
  providerStarts,
  savings,
  savingsPeriod,
  type PlaceFigures,
  type ProviderFigures,
} from "./savings.ts";
import { ownMachines, tiers } from "./tiers.ts";

export type Alternative = "blacksmith" | "github";

export type ReportRange = "30" | "month" | "last";

export type ReportFigures = {
  range: ReportRange;
  /** The first and the last included UTC day, such as "1 November". */
  period: string;
  /** The jobs of the own machines. */
  own: PlaceFigures;
  /** A row for each provider that ran jobs in the range. */
  providers: readonly ProviderFigures[];
  /** The known cost of the own machines, or null when none is set. */
  ownCost: number | null;
  /** Some machine costs are missing, so ownCost is the known part. */
  ownPartial?: true;
  planCost: number;
  /** Some plan prices are not known, so planCost is the known part. */
  planPartial?: true;
  alternatives: Readonly<Record<Alternative, number | null>>;
  /** An alternative without a complete price, and the size it lacks. */
  priceGap?: { alternative: Alternative; size: string };
  ownLabel: string;
  planLabel: string;
  /** The starts of each day, on own machines and on the providers together. */
  daily: {
    labels: readonly string[];
    own: readonly number[];
    overflow: readonly number[];
  };
  /** The report still refreshes, so the page reads it again. */
  updating?: true;
};

/** The jobs of the range, on own machines and on every provider. */
export function jobCount(figures: ReportFigures): number {
  return figures.own.starts + providerStarts(figures.providers);
}

/** The complete cost with ciproxy, or null while a cost is missing. */
export function totalCost(figures: ReportFigures): number | null {
  if (figures.ownCost === null || figures.ownPartial || figures.planPartial) {
    return null;
  }
  return figures.ownCost + providerCost(figures.providers) + figures.planCost;
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
  own: savings.own,
  providers: savings.providers,
  ownCost: savings.ownCost,
  planCost: savings.planCost,
  alternatives: savings.alternatives,
  ownLabel: ownMachines(3),
  planLabel: `ciproxy ${tiers.team.name}`,
  daily: dailyStarts,
};

const emptyFigures: ReportFigures = {
  ...known,
  own: { starts: 0, minutes: 0 },
  providers: [],
  alternatives: { blacksmith: 0, github: 0 },
  daily: { labels: [], own: [], overflow: [] },
};

/** The overflow of ciproxy-large on Second provider, from 9 November. */
const secondShare = { starts: 412, minutes: 2_118, cost: 17.4 };

/** The provider rows of the what-if with two providers. */
function withSecond(boat: ProviderFigures): readonly ProviderFigures[] {
  return [
    {
      provider: "boat",
      starts: boat.starts - secondShare.starts,
      minutes: boat.minutes - secondShare.minutes,
      cost: Math.round((boat.cost - secondShare.cost) * 100) / 100,
    },
    { provider: "second", ...secondShare },
  ];
}

/** Sunday 1 November: the first day of the month, quiet as a weekend. */
const quietDay = formatDay(new Date(Date.UTC(2026, 10, 1)));

export const savingsFigures = {
  report: known,
  "missing-costs": { ...known, ownCost: null },
  negative: {
    ...known,
    range: "month",
    period: quietDay,
    own: { starts: 102, minutes: 498 },
    providers: [{ provider: "boat", starts: 7, minutes: 41, cost: 0.16 }],
    ownCost: (3 * ax42.monthlyCost) / 30,
    planCost: tiers.team.price / 30,
    alternatives: { blacksmith: 5.86, github: 8.62 },
    daily: { labels: ["1"], own: [102], overflow: [7] },
  },
  "missing-price": {
    ...known,
    alternatives: { blacksmith: null, github: savings.alternatives.github },
    priceGap: { alternative: "blacksmith", size: "16 vCPU and 64 GB" },
  },
  "partial-costs": {
    ...known,
    ownCost: (savings.ownCost * 2) / 3,
    ownPartial: true,
  },
  "unknown-costs": {
    ...known,
    planCost: (savings.planCost * 20) / 30,
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
  "two-providers": {
    ...known,
    providers: savings.providers.flatMap(withSecond),
  },
  loading: known,
  error: known,
} as const satisfies Readonly<Record<string, ReportFigures>>;

export type SavingsState = keyof typeof savingsFigures;
