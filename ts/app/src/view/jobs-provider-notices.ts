// The notices of a provider hold: the spend limit, a rejected key, billing on
// the provider account, and a refusal with a retry time that is after the
// read. Each notice names its provider, and a spend limit or a refusal says
// when the provider takes jobs again, from the facts of the hold at the read.
// While a provider takes no new job, a waiting job goes on to the next place
// of its order. When no other provider starts a waiting job now, a job can
// start only on an own machine that runs its label. So a notice says that
// every waiting job waits for a free own slot only when each waiting label
// fits the own machines. Otherwise it says so only of the jobs whose label
// fits them, and it promises the other jobs no place. When another provider
// starts waiting jobs, the notice says that jobs skip the held provider. A
// refused size holds only the labels that use it, so it raises no notice here.

import type { JobOverflow } from "../api/jobs-types.ts";
import type { OverflowProvider, ProviderHold } from "../api/schema.ts";
import { addUtcDays, formatCents } from "./exact-format.ts";
import {
  againText,
  type HoldFacts,
  type HoldFactsOf,
} from "./jobs-hold-facts.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { LabelPlace } from "./jobs-places.ts";
import { providerName } from "./providers.ts";

const monthName = new Intl.DateTimeFormat("en-GB", {
  month: "long",
  timeZone: "UTC",
});

/** Where the waiting jobs wait during a hold of one provider. */
type HoldWords = { wait: string; take: string };

function holdWords(name: string, places: readonly LabelPlace[]): HoldWords {
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

/** The notice of one hold of the provider of `overflow`, from its facts. */
function holdNotice(
  facts: HoldFacts,
  overflow: JobOverflow,
  words: HoldWords,
  manage: (label: string) => NoticeAction,
): LiveNotice | null {
  const { provider } = facts;
  const name = providerName(provider);
  switch (facts.hold) {
    case "spend_limit": {
      const month = monthName.format(new Date(addUtcDays(facts.until, -1)));
      const limit = overflow.limit_cents ?? null;
      return {
        key: `${provider}-spend-limit`,
        tone: "warning",
        title: limit
          ? `${name} reached its ${formatCents(limit)} limit for ${month}`
          : `${name} reached its spend limit for ${month}`,
        body: `${words.wait}. ${againText(facts)}.`,
        action: manage("Raise the limit"),
      };
    }
    case "key_rejected":
      return {
        key: `${provider}-key`,
        tone: "warning",
        title: `${name} rejected the key`,
        body: `${words.wait}, until a new key is connected.`,
        action: manage("Connect a new key"),
      };
    case "billing_required":
      return {
        key: `${provider}-billing`,
        tone: "warning",
        title: `${name} needs billing on your ${name} account`,
        body: `${name} starts no new sandboxes until its billing is set up. ${words.wait}.`,
        action: null,
      };
    case "retry_after":
      return {
        key: `${provider}-refused`,
        tone: "warning",
        title: `${name} refused new sandboxes`,
        body: `${words.take}, and ${againText(facts)}.`,
        action: null,
      };
    default:
      return null;
  }
}

export function providerNotices(
  overflow: JobOverflow,
  facts: HoldFactsOf,
  places: readonly LabelPlace[],
  manage: (label: string) => NoticeAction,
): HoldNotice[] {
  const { provider } = overflow;
  const words = holdWords(providerName(provider), places);
  // The holds of every job, in the order of their notices. The facts drop a
  // refusal whose retry time is not after the read.
  const holds: ProviderHold[] = [];
  if (overflow.limit_reached === true) holds.push("spend_limit");
  if (overflow.key_rejected) holds.push("key_rejected");
  if (overflow.billing_required) holds.push("billing_required");
  holds.push("retry_after");
  return holds.flatMap((hold) => {
    const held = facts(provider, hold);
    const notice =
      held === null ? null : holdNotice(held, overflow, words, manage);
    return notice === null ? [] : [{ provider, hold, notice }];
  });
}
