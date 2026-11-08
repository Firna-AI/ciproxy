// What Setup and Settings under Overflow show of Boat. Every fact comes from
// the API: the account name, the rejection time, the spend parts, whether
// the limit is reached, and the first day of the next month in UTC, with the
// budget month before it. A reached limit comes only from the API flag,
// never from rounded amounts.

import type {
  BoatAccountView,
  BoatWallet,
  OverflowSettings,
} from "../api/schema.ts";
import type { OverflowSnapshot } from "../state/overflow.ts";
import { formatDay } from "./format.ts";
import { formatCents, shareOf } from "./money.ts";
import type { NoticeModel } from "./page-model.ts";
import type { BoatProgress } from "./setup-steps.ts";

/** The overflow waits of job-routing.md, in seconds. */
export const waitChoices = [0, 15, 30, 60, 120] as const;

/** "At once", "15 seconds", "1 minute", "2 minutes". */
export function waitLabel(seconds: number): string {
  if (seconds === 0) return "At once";
  if (seconds === 60) return "1 minute";
  if (seconds > 60 && seconds % 60 === 0) return `${seconds / 60} minutes`;
  return `${seconds} seconds`;
}

export type WaitOption = { label: string; value: string };

/** The choices, with a saved wait that is not one of them kept visible. */
export function waitOptions(saved: number): WaitOption[] {
  const values: number[] = [...waitChoices];
  if (!values.includes(saved)) values.push(saved);
  return values
    .sort((a, b) => a - b)
    .map((seconds) => ({ label: waitLabel(seconds), value: String(seconds) }));
}

/** The actions of an account: every account can disconnect. */
type AccountActions<ChangeKey extends boolean> = {
  disconnect: true;
  /** Only an accepted key changes; a rejected key reconnects in the form. */
  changeKey: ChangeKey;
};

/** The Boat panel of Settings under Overflow, by account and key state. */
export type BoatPanelModel =
  | { kind: "none" }
  | { kind: "accepted"; name: string; actions: AccountActions<true> }
  | { kind: "rejected"; name: string; actions: AccountActions<false> };

export function boatPanelModel(settings: OverflowSettings): BoatPanelModel {
  const account = settings.account ?? null;
  if (account === null) return { kind: "none" };
  if (account.rejected_at) {
    return {
      kind: "rejected",
      name: account.name,
      actions: { disconnect: true, changeKey: false },
    };
  }
  return {
    kind: "accepted",
    name: account.name,
    actions: { disconnect: true, changeKey: true },
  };
}

/** The identity of the personal wallet; each organization uses its Boat ID. */
export const personalWallet = "personal";

/** One wallet as a radio card of the wallet choice. */
export type WalletOption = { value: string; title: string; body: string };

/** The wallets of a checked key, in the order of the API answer. */
export function walletOptions(wallets: readonly BoatWallet[]): WalletOption[] {
  return wallets.map((item) => ({
    value: item.wallet,
    title: item.name,
    body:
      item.wallet === personalWallet
        ? "Personal wallet"
        : "Boat organization wallet",
  }));
}

/** The account that works now: connected, and its key not rejected. */
export function activeAccount(
  settings: OverflowSettings,
): BoatAccountView | null {
  const account = settings.account ?? null;
  return account !== null && !account.rejected_at ? account : null;
}

export type SpendFigures = {
  confirmed: string;
  unconfirmed: string;
  /** The part of the unconfirmed costs whose lifetime ended, if any. */
  settled: string | null;
  limit: string;
  /** Shares of the limit for the bar, from 0 to 1, together at most 1. */
  confirmedShare: number;
  unconfirmedShare: number;
  /** Only the API flag sets this. */
  reached: boolean;
};

/** The spend of the month, or null when the API could not read it. */
export function spendFigures(settings: OverflowSettings): SpendFigures | null {
  const spend = settings.spend ?? null;
  const limit = settings.spend_limit_cents ?? null;
  if (spend === null || limit === null) return null;
  const confirmedShare = shareOf(spend.confirmed_cents, limit);
  return {
    confirmed: formatCents(spend.confirmed_cents),
    unconfirmed: formatCents(spend.unconfirmed_cents),
    settled:
      BigInt(spend.settled_bound_cents) > 0n
        ? formatCents(spend.settled_bound_cents)
        : null,
    limit: formatCents(limit),
    confirmedShare,
    unconfirmedShare: Math.min(
      shareOf(spend.unconfirmed_cents, limit),
      1 - confirmedShare,
    ),
    reached: spend.limit_reached === true,
  };
}

/** "November": the UTC month before `nextMonthAt`, whose budget counts. */
function budgetMonth(nextMonthAt: string): string {
  const next = new Date(nextMonthAt);
  const month = Date.UTC(next.getUTCFullYear(), next.getUTCMonth() - 1, 1);
  return new Intl.DateTimeFormat("en-GB", {
    month: "long",
    timeZone: "UTC",
  }).format(month);
}

/** The one notice of the page: a rejected key, a reached limit, or costs. */
export function overflowNotice(settings: OverflowSettings): NoticeModel | null {
  const rejected = settings.account?.rejected_at ?? null;
  if (rejected !== null) return rejectedNotice(rejected);
  const spend = settings.spend ?? null;
  const limit = settings.spend_limit_cents ?? null;
  if (spend?.limit_reached === true && limit !== null) {
    return {
      tone: "warning",
      title: `Boat spend reached the ${formatCents(limit)} limit for ${budgetMonth(settings.next_month_at)}`,
      body: `Jobs run on your machines only, and wait there for a free slot. Boat takes jobs again on ${formatDay(settings.next_month_at, "UTC")}, or when you raise the limit.`,
    };
  }
  if (spend !== null && !spend.bounded) {
    return {
      tone: "warning",
      title: "Boat costs are not known yet",
      body: "Jobs run on your machines only until Boat reports the cost of each start.",
    };
  }
  return null;
}

export function rejectedNotice(rejectedAt: string): NoticeModel {
  return {
    tone: "danger",
    title: `Boat rejected the API key on ${formatDay(rejectedAt)}`,
    body: "Jobs run on your machines only until you connect Boat with a new key.",
  };
}

/** The line of a done Boat step in Setup. */
export function connectedSummary(settings: OverflowSettings): string | null {
  const account = activeAccount(settings);
  if (account === null) return null;
  const limit = settings.spend_limit_cents ?? null;
  const parts = [`Connected to the Boat account ${account.name}`];
  if (limit !== null) parts.push(`spend limit ${formatCents(limit)} a month`);
  return parts.join(" · ");
}

/**
 * What Setup knows of Boat. A skip counts only for the person who skipped,
 * and only while no account is connected.
 */
export function boatProgress(
  snapshot: OverflowSnapshot,
  skippedBy: string | null,
  login: string,
): BoatProgress {
  const connected =
    snapshot.status === "ready" ? connectedSummary(snapshot.settings) : null;
  return { connected, skipped: connected === null && skippedBy === login };
}
