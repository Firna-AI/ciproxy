// The Overflow panel of the Machines list, from the overflow summary that
// every role reads: where jobs go when the machines are busy, and the Boat
// spend of the month against its limit. The confirmed spend fills the meter;
// the amount that Boat has not confirmed shows beside it.

import type { OverflowSettings } from "../api/machine-schema.ts";
import { overflowCopy } from "./machines-copy.ts";
import { formatCents, formatDay, waitText } from "./machines-format.ts";

export type SpendMeter = {
  /** The confirmed spend against the limit, from 0 to 1. */
  ratio: number;
  value: string;
  detail: string | null;
};

export type OverflowModel = {
  caption: string;
  meter: SpendMeter | null;
  /** Set when the account is connected but its spend is not known yet. */
  note: string | null;
};

function ratio(spent: string, limit: string | null): number {
  if (limit === null) return 0;
  const max = Number(limit);
  if (max <= 0) return Number(spent) > 0 ? 1 : 0;
  return Math.min(Math.max(Number(spent) / max, 0), 1);
}

function meter(summary: OverflowSettings): SpendMeter | null {
  const spend = summary.spend ?? null;
  if (spend === null) return null;
  const limit = summary.spend_limit_cents ?? null;
  const spent = formatCents(spend.confirmed_cents);
  return {
    ratio: ratio(spend.confirmed_cents, limit),
    value:
      limit === null ? spent : overflowCopy.spentOf(spent, formatCents(limit)),
    detail:
      BigInt(spend.unconfirmed_cents) > BigInt(0)
        ? overflowCopy.pending(formatCents(spend.unconfirmed_cents))
        : null,
  };
}

function caption(summary: OverflowSettings, account: string): string {
  if (!summary.enabled) return overflowCopy.off;
  if (summary.spend?.limit_reached === true) {
    const limit = summary.spend_limit_cents ?? null;
    return overflowCopy.limit(
      limit === null ? null : formatCents(limit),
      formatDay(summary.next_month_at),
    );
  }
  return overflowCopy.connected(account, waitText(summary.wait_seconds));
}

export function overflowModel(summary: OverflowSettings): OverflowModel {
  const account = summary.account ?? null;
  if (!summary.available) {
    return { caption: overflowCopy.machinesOnly, meter: null, note: null };
  }
  if (account === null) {
    return { caption: overflowCopy.notConnected, meter: null, note: null };
  }
  const spend = meter(summary);
  const rejected = account.rejected_at ?? null;
  return {
    caption:
      rejected === null
        ? caption(summary, account.name)
        : overflowCopy.rejected(formatDay(rejected)),
    meter: spend,
    note: spend === null ? overflowCopy.noSpend : null,
  };
}
