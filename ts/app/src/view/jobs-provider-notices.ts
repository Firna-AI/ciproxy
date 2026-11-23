// The notices of a provider hold: the spend limit, a rejected key, billing on
// the provider account, and a refusal with a retry time that is after the
// read. Each notice names its provider. While a provider takes no new job, a
// waiting job goes on to the next place of its order. When no other provider
// starts a waiting job now, a job can start only on an own machine that runs
// its label. So a notice says that every waiting job waits for a free own
// slot only when each waiting label fits the own machines. Otherwise it says
// so only of the jobs whose label fits them, and it promises the other jobs
// no place. When another provider starts waiting jobs, the notice says that
// jobs skip the held provider. A refused size holds only the labels that use
// it, so it raises no notice here.

import type { JobOverflow } from "../api/jobs-types.ts";
import type { OverflowProvider, ProviderHold } from "../api/schema.ts";
import { addUtcDays, formatCents, formatUtcDate } from "./exact-format.ts";
import { formatTime } from "./format.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { LabelPlace, ProviderHoldState } from "./jobs-places.ts";
import { providerName } from "./providers.ts";

const monthName = new Intl.DateTimeFormat("en-GB", {
  month: "long",
  timeZone: "UTC",
});

/** Where the waiting jobs wait during a hold of one provider. */
function holdWords(name: string, places: readonly LabelPlace[]) {
  const othersStart = places.some((place) =>
    place.providers.some((item) => item.start.kind === "starts"),
  );
  if (othersStart) {
    const skip = `Jobs skip ${name} and go on to the next place of their label`;
    return { wait: skip, take: skip };
  }
  if (places.every((place) => place.own === "fits")) {
    return {
      wait: "Jobs run on your machines only, and wait there for a free slot",
      take: "Waiting jobs take the first free slot on your machines",
    };
  }
  return {
    wait: "Jobs whose label fits your machines wait there for a free slot",
    take: "Waiting jobs whose label fits your machines take the first free slot there",
  };
}

/** A notice of a provider hold, with the hold that it states. */
export type HoldNotice = {
  provider: OverflowProvider;
  hold: ProviderHold;
  notice: LiveNotice;
};

export function providerNotices(
  overflow: JobOverflow,
  hold: ProviderHoldState,
  places: readonly LabelPlace[],
  manage: (label: string) => NoticeAction,
): HoldNotice[] {
  const { provider } = overflow;
  const name = providerName(provider);
  const words = holdWords(name, places);
  const notices: HoldNotice[] = [];
  const push = (kind: ProviderHold, notice: LiveNotice) =>
    notices.push({ provider, hold: kind, notice });
  if (overflow.limit_reached === true) {
    const until = overflow.next_month.slice(0, 10);
    const month = monthName.format(new Date(addUtcDays(until, -1)));
    const limit = overflow.limit_cents ?? null;
    push("spend_limit", {
      key: `${provider}-spend-limit`,
      tone: "warning",
      title: limit
        ? `${name} reached its ${formatCents(limit)} limit for ${month}`
        : `${name} reached its spend limit for ${month}`,
      body: `${words.wait}. ${name} takes jobs again on ${formatUtcDate(until)}.`,
      action: manage("Raise the limit"),
    });
  }
  if (overflow.key_rejected) {
    push("key_rejected", {
      key: `${provider}-key`,
      tone: "warning",
      title: `${name} rejected the key`,
      body: `${words.wait}, until a new key is connected.`,
      action: manage("Connect a new key"),
    });
  }
  if (overflow.billing_required) {
    push("billing_required", {
      key: `${provider}-billing`,
      tone: "warning",
      title: `${name} needs billing on your ${name} account`,
      body: `${name} starts no new sandboxes until its billing is set up. ${words.wait}.`,
      action: null,
    });
  }
  if (hold.retryUntil !== null) {
    push("retry_after", {
      key: `${provider}-refused`,
      tone: "warning",
      title: `${name} refused new sandboxes`,
      body: `${words.take}, and ciproxy tries ${name} again at ${formatTime(hold.retryUntil)}.`,
      action: null,
    });
  }
  return notices;
}
