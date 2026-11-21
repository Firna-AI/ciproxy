// The Boat spend of mokly-ai in November, apart by what Boat confirmed and
// what it has not confirmed yet, as "Spend limit" of overflow.md says. A cost
// that Boat has not confirmed counts at its bound: the hourly price of its
// size for the whole sandbox lifetime, plus the highest start fee. A bound
// whose lifetime ended without a report from Boat stays counted, and it stays
// unconfirmed. The two parts add up to the spend of the month in machines.ts.

import { boat } from "./machines.ts";

export type BoatSpend = {
  /** Costs that Boat reported, in US dollars. */
  confirmed: number;
  /** Bounds of starts whose cost Boat has not reported yet. */
  unconfirmed: number;
  /** The part of `unconfirmed` whose lifetime ended without a report. */
  settled: number;
};

function cents(value: number): number {
  return Math.round(value * 100) / 100;
}

function split(total: number, confirmed: number, settled: number): BoatSpend {
  return { confirmed, unconfirmed: cents(total - confirmed), settled };
}

export const spendScenes = {
  /** 18 November, the busy day of the release peak. */
  steady: split(boat.spentThisMonth, 48.1, 1.2),
  /** 26 November: the counted spend reached the limit. */
  limitReached: split(boat.spendLimit, 186.4, 1.2),
} as const satisfies Record<string, BoatSpend>;
