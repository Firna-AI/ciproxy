// The savings report as a model: the range, the savings figure, the cost
// bars, the price-list note, and the starts of each day. Every amount comes
// from the API. The figure shows only when the API names the cheaper
// alternative and gives its signed saving, and its words keep the sign: a
// negative saving says "more than". The shown end of a range is the day
// before its first excluded day.

import type {
  SavingsAlternative,
  SavingsComparison,
  SavingsPage,
  SavingsRange,
} from "../api/savings-types.ts";
import { workspaceHref } from "../routing/pages.ts";
import {
  absoluteCents,
  centsSign,
  formatCents,
  formatCount,
  formatUtcDate,
  formatUtcRange,
} from "./exact-format.ts";
import { joinWords } from "./format.ts";

export const rangeLabels: Readonly<Record<SavingsRange, string>> = {
  last30_days: "Last 30 days",
  this_month: "This month",
  last_month: "Last month",
};

export const rangeOrder: readonly SavingsRange[] = [
  "last30_days",
  "this_month",
  "last_month",
];

export const alternativeNames: Readonly<Record<SavingsAlternative, string>> = {
  blacksmith: "Blacksmith",
  github: "GitHub-hosted runners",
};

/** "Last 30 days · 20 October – 18 November". */
export function savingsSubtitle(page: SavingsPage): string {
  return `${rangeLabels[page.range]} · ${formatUtcRange(page.from, page.until)}`;
}

/** The starts of the range on own machines and on Boat together. */
export function totalStarts(page: SavingsPage): bigint {
  return BigInt(page.machines.starts) + BigInt(page.boat.starts);
}

export type Hero = { figure: string; text: string };

function words(cents: string): string {
  const sign = centsSign(cents);
  if (sign < 0) return "more than";
  return sign > 0 ? "less than" : "the same as";
}

function comparison(
  page: SavingsPage,
  alternative: SavingsAlternative,
): SavingsComparison | null {
  return (
    page.comparisons.find((item) => item.alternative === alternative) ?? null
  );
}

/**
 * "$390.50" and "less than Blacksmith for the same 17,760 jobs, and $737.50
 * less than GitHub-hosted runners." Null while a cost is missing.
 */
export function savingsHero(page: SavingsPage): Hero | null {
  const lead = page.lead ?? null;
  if (lead === null || (page.total_cents ?? null) === null) return null;
  const first = comparison(page, lead)?.savings_cents ?? null;
  if (first === null) return null;
  const other = lead === "blacksmith" ? "github" : "blacksmith";
  const second = comparison(page, other)?.savings_cents ?? null;
  const jobs = formatCount(totalStarts(page));
  const tail =
    second === null
      ? ""
      : `, and ${formatCents(absoluteCents(second))} ${words(second)} ${alternativeNames[other]}`;
  return {
    figure: formatCents(absoluteCents(first)),
    text: `${words(first)} ${alternativeNames[lead]} for the same ${jobs} jobs${tail}.`,
  };
}

export type CostBar = {
  key: string;
  label: string;
  value: string;
  /** The length of the bar against the longest bar, from 0 to 1. */
  share: number;
  accent: boolean;
};

/** The known costs of the same jobs. An unknown cost has no bar. */
export function costBars(page: SavingsPage): CostBar[] {
  const known = [
    { key: "ciproxy", label: "With ciproxy", cents: page.total_cents ?? null },
    {
      key: "blacksmith",
      label: "Blacksmith",
      cents: comparison(page, "blacksmith")?.cost_cents ?? null,
    },
    {
      key: "github",
      label: "GitHub-hosted",
      cents: comparison(page, "github")?.cost_cents ?? null,
    },
  ].flatMap((bar) =>
    bar.cents === null ? [] : [{ ...bar, cents: bar.cents }],
  );
  const longest = Math.max(1, ...known.map((bar) => Number(bar.cents)));
  return known.map((bar) => ({
    key: bar.key,
    label: bar.label,
    value: formatCents(bar.cents),
    share: Math.max(0, Number(bar.cents)) / longest,
    accent: bar.key === "ciproxy",
  }));
}

/** The note that names the dates of the price lists that the report uses. */
export function priceListNote(page: SavingsPage): string {
  const days = [
    ...new Set(page.comparisons.map((item) => item.price_list_date)),
  ]
    .sort()
    .map((day) => formatUtcDate(day, true));
  return `Blacksmith and GitHub-hosted costs use their list prices per minute for the same VM sizes, from the price lists of ${joinWords(days, "and")}.`;
}

export type StartsSeries = {
  /** The day of the month of each UTC date. */
  days: string[];
  machines: number[];
  boat: number[];
};

export function startsSeries(page: SavingsPage): StartsSeries {
  return {
    days: page.days.map((item) => String(Number(item.day.slice(8, 10)))),
    machines: page.days.map((item) => Number(item.machines.starts)),
    boat: page.days.map((item) => Number(item.boat.starts)),
  };
}

export type EmptyReport = { title: string; body: string; setup: boolean };

/** The report of a range with no start. */
export function emptyReport(range: SavingsRange): EmptyReport {
  if (range === "last30_days") {
    return {
      title: "Your savings report starts with your first job",
      body: "It compares what your jobs cost with the same jobs on GitHub-hosted runners and on Blacksmith.",
      setup: true,
    };
  }
  return {
    title: `No jobs ran ${rangeLabels[range].toLowerCase()}`,
    body: "Choose another range to see your savings.",
    setup: false,
  };
}

/** The range of the address, or the last 30 days for a missing value. */
export function rangeOf(
  value: string | readonly string[] | undefined,
): SavingsRange {
  return value === "this_month" || value === "last_month"
    ? value
    : "last30_days";
}

/** The address of Savings for one range. The last 30 days is the default. */
export function savingsHref(workspaceId: string, range: SavingsRange): string {
  const base = workspaceHref("savings", workspaceId);
  return range === "last30_days" ? base : `${base}&range=${range}`;
}
