// The Overflow panel of the Machines list, from the overflow summary that
// every role reads: a row for each connected provider, with its account and
// the spend of the month against its own limit, the confirmed and the
// unconfirmed costs apart, and the overflow wait under the rows, because it
// holds for every provider. A row says why its provider takes no jobs now,
// when the API says so. With no connected provider, the panel says that jobs
// run on the machines only.

import type {
  OverflowProvider,
  OverflowSettings,
  ProviderSettings,
} from "../api/schema.ts";
import { overflowCopy } from "./machines-copy.ts";
import { formatCents, formatDay, waitText } from "./machines-format.ts";
import {
  connectedProviders,
  spendFigures,
  type SpendFigures,
} from "./overflow.ts";
import { providerName } from "./providers.ts";

export type ProviderRow = {
  provider: OverflowProvider;
  name: string;
  /** "Connected to the account mokly". */
  account: string;
  /** Why the provider takes no jobs now, when the API says so. */
  hold: string | null;
  /** The spend of the month, or null while the provider has not reported it. */
  spend: SpendFigures | null;
  /** Set when the spend is not known yet. */
  note: string | null;
};

export type OverflowModel =
  | { kind: "none" }
  | {
      kind: "providers";
      rows: readonly ProviderRow[];
      /** The overflow wait, or the switch that holds every provider. */
      caption: string;
    };

function holdLine(
  summary: OverflowSettings,
  provider: ProviderSettings,
): string | null {
  const name = providerName(provider.provider);
  const rejected = provider.account?.rejected_at ?? null;
  if (rejected !== null) {
    return overflowCopy.rejected(name, formatDay(rejected));
  }
  if (!provider.available) return overflowCopy.unavailable(name);
  if (provider.spend?.limit_reached === true) {
    const limit = provider.spend_limit_cents ?? null;
    return overflowCopy.limit(
      name,
      limit === null ? null : formatCents(limit),
      formatDay(summary.next_month_at),
    );
  }
  return null;
}

function row(
  summary: OverflowSettings,
  provider: ProviderSettings,
): ProviderRow {
  const name = providerName(provider.provider);
  const spend = spendFigures(provider);
  return {
    provider: provider.provider,
    name,
    account: overflowCopy.connected(provider.account?.name ?? ""),
    hold: holdLine(summary, provider),
    spend,
    note: spend === null ? overflowCopy.noSpend(name) : null,
  };
}

export function overflowModel(summary: OverflowSettings): OverflowModel {
  const connected = connectedProviders(summary);
  if (connected.length === 0) return { kind: "none" };
  return {
    kind: "providers",
    rows: connected.map((provider) => row(summary, provider)),
    caption: summary.enabled
      ? overflowCopy.wait(summary.wait_seconds, waitText(summary.wait_seconds))
      : overflowCopy.off,
  };
}
