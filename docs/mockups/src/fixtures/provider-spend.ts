// The spend of each provider account of mokly-ai in November, apart by what
// the provider confirmed and what it has not confirmed yet, as "What the
// limit counts" of overflow-spend.md says. A cost that a provider has not
// confirmed counts at its bound: the hourly price of its size for the whole
// sandbox lifetime, plus the highest start fee on Boat. A bound whose
// lifetime ended without a report stays counted, and it stays unconfirmed.
// The two parts add up to the spend of the month in overflow.ts. In the
// what-ifs with two providers, Second provider took a part of the spend that
// Boat has in the main story.

import { boat, secondProvider } from "./overflow.ts";

export type ProviderSpend = {
  /** Costs that the provider reported, in US dollars. */
  confirmed: number;
  /** Bounds of starts whose cost the provider has not reported yet. */
  unconfirmed: number;
  /** The part of `unconfirmed` whose lifetime ended without a report. */
  settled: number;
};

function cents(value: number): number {
  return Math.round(value * 100) / 100;
}

function split(
  total: number,
  confirmed: number,
  settled: number,
): ProviderSpend {
  return { confirmed, unconfirmed: cents(total - confirmed), settled };
}

export const spendScenes = {
  /** 18 November, the busy day of the release peak. */
  steady: split(boat.spentThisMonth, 48.1, 1.2),
  /** 26 November: the counted Boat spend reached the limit. */
  limitReached: split(boat.spendLimit, 186.4, 1.2),
  /** 18 November with two providers: the Boat part of the spend. */
  boatBesideSecond: split(
    cents(boat.spentThisMonth - secondProvider.spentThisMonth),
    33.6,
    1.2,
  ),
  /** 18 November with two providers: the Second provider part. */
  second: split(secondProvider.spentThisMonth, 15.1, 0),
  /** 24 November with two providers: Second provider reached its limit. */
  secondLimitReached: split(secondProvider.spendLimit, 93.2, 0),
} as const satisfies Record<string, ProviderSpend>;
