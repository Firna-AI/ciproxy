// What Setup and Settings under Overflow show of each provider. Every fact
// comes from the API: the providers, whether each is available, its account
// name, the rejection time, the spend parts, whether the limit is reached,
// and the first day of the next month in UTC, with the budget month before
// it. A reached limit comes only from the API flag, never from rounded
// amounts. Each notice names its provider, and says when jobs start there
// again; the other places of a label order go on as before.

import type {
  OverflowSettings,
  ProviderAccountView,
  ProviderSettings,
} from "../api/schema.ts";
import type { OverflowSnapshot } from "../state/overflow.ts";
import { formatDay } from "./format.ts";
import { formatCents, shareOf } from "./money.ts";
import type { NoticeModel } from "./page-model.ts";
import { providerName, providerNames } from "./providers.ts";
import type { ProviderProgress } from "./setup-steps.ts";

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

/** The providers that have an account, also one whose key was rejected. */
export function connectedProviders(
  settings: OverflowSettings,
): ProviderSettings[] {
  return settings.providers.filter((provider) => provider.account != null);
}

/**
 * The panels of Settings under Overflow: one for each provider that is
 * connected or available. A provider that the deployment cannot start and
 * that has no account gets no panel.
 */
export function panelProviders(settings: OverflowSettings): ProviderSettings[] {
  return settings.providers.filter(
    (provider) => provider.account != null || provider.available,
  );
}

/** The actions of an account: every account can disconnect. */
type AccountActions = {
  disconnect: true;
  /** A new key can connect: the key is accepted and the provider available. */
  changeKey: boolean;
};

/** The panel of one provider, by its account and key state. */
export type ProviderPanelModel =
  | { kind: "none" }
  | { kind: "accepted"; name: string; actions: AccountActions }
  | { kind: "rejected"; name: string; actions: AccountActions };

export function providerPanelModel(
  provider: ProviderSettings,
): ProviderPanelModel {
  const account = provider.account ?? null;
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
    actions: { disconnect: true, changeKey: provider.available },
  };
}

/** The account that works now: connected, and its key not rejected. */
export function activeAccount(
  provider: ProviderSettings,
): ProviderAccountView | null {
  const account = provider.account ?? null;
  return account !== null && !account.rejected_at ? account : null;
}

export type SpendFigures = {
  /** The provider that reports the costs. */
  name: string;
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
export function spendFigures(provider: ProviderSettings): SpendFigures | null {
  const spend = provider.spend ?? null;
  const limit = provider.spend_limit_cents ?? null;
  if (spend === null || limit === null) return null;
  const confirmedShare = shareOf(spend.confirmed_cents, limit);
  return {
    name: providerName(provider.provider),
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

/** The notice of a key that the provider stopped accepting. */
export function rejectedNotice(
  provider: ProviderSettings,
  rejectedAt: string,
): NoticeModel {
  const name = providerName(provider.provider);
  return {
    tone: "danger",
    title: `${name} rejected the API key on ${formatDay(rejectedAt)}`,
    body: `ciproxy starts no new jobs on ${name} until you connect a new key.`,
  };
}

/**
 * The notice of one provider: a rejected key, an unavailable provider that
 * keeps its account, a reached limit, or costs that have no safe count.
 */
function providerNotice(
  settings: OverflowSettings,
  provider: ProviderSettings,
): NoticeModel | null {
  const account = provider.account ?? null;
  if (account === null) return null;
  const name = providerName(provider.provider);
  if (account.rejected_at) return rejectedNotice(provider, account.rejected_at);
  if (!provider.available) {
    return {
      tone: "warning",
      title: `${name} is unavailable right now`,
      body: `ciproxy starts no new jobs on ${name} until it is available again. Jobs skip ${name} and go on to the next place of their label.`,
    };
  }
  const spend = provider.spend ?? null;
  const limit = provider.spend_limit_cents ?? null;
  if (spend?.limit_reached === true && limit !== null) {
    return {
      tone: "warning",
      title: `${name} spend reached the ${formatCents(limit)} limit for ${budgetMonth(settings.next_month_at)}`,
      body: `ciproxy starts no new jobs on ${name} until ${formatDay(settings.next_month_at, "UTC")}, or until you raise the limit.`,
    };
  }
  if (spend !== null && !spend.bounded) {
    return {
      tone: "warning",
      title: `${name} costs are not known yet`,
      body: `ciproxy starts no new jobs on ${name} until ${name} reports the cost of each start.`,
    };
  }
  return null;
}

/** The notices of the page, one for each provider that needs one. */
export function overflowNotices(settings: OverflowSettings): NoticeModel[] {
  return settings.providers.flatMap((provider) => {
    const notice = providerNotice(settings, provider);
    return notice === null ? [] : [notice];
  });
}

/** The line of a done provider step in Setup, or null with no active account. */
export function connectedSummary(settings: OverflowSettings): string | null {
  const active = settings.providers.flatMap((provider) => {
    const account = activeAccount(provider);
    return account === null ? [] : [{ provider, account }];
  });
  const [first] = active;
  if (first === undefined) return null;
  if (active.length > 1) {
    const names = active.map((item) => item.provider.provider);
    return `Connected to ${providerNames(names, "and")}`;
  }
  const name = providerName(first.provider.provider);
  const parts = [`Connected to the ${name} account ${first.account.name}`];
  const limit = first.provider.spend_limit_cents ?? null;
  if (limit !== null) parts.push(`spend limit ${formatCents(limit)} a month`);
  return parts.join(" · ");
}

/**
 * What Setup knows of the providers. A skip counts only for the person who
 * skipped, and only while no provider has an active account.
 */
export function providerProgress(
  snapshot: OverflowSnapshot,
  skippedBy: string | null,
  login: string,
): ProviderProgress {
  const connected =
    snapshot.status === "ready" ? connectedSummary(snapshot.settings) : null;
  return { connected, skipped: connected === null && skippedBy === login };
}
