// The facts of each provider hold at the time of the read, which both kinds
// of notice of a hold read: the notice of a provider for every job, and the
// notice of a waiting label that waits for the hold. A spend limit holds new
// starts until the first UTC day of the next month, or until the limit
// rises. A refusal holds them until its saved retry time, and only while
// that time is after the read, as the overflow service waits only until
// then. So both notices give the same date or time, and no notice states a
// refusal whose retry time is not after the read.

import type { JobsLivePage } from "../api/jobs-types.ts";
import type { OverflowProvider, ProviderHold } from "../api/schema.ts";
import { formatUtcDate } from "./exact-format.ts";
import { formatTime } from "./format.ts";
import { observation, providerHold } from "./jobs-places.ts";
import { providerName } from "./providers.ts";

/** A hold of a provider at the read time, with its end when the API gives it. */
export type HoldFacts =
  | {
      provider: OverflowProvider;
      hold: "spend_limit";
      /** The first UTC day of the next month, such as "2026-12-01". */
      until: string;
    }
  | {
      provider: OverflowProvider;
      hold: "retry_after";
      /** The saved retry time, which is after the read. */
      until: string;
    }
  | {
      provider: OverflowProvider;
      hold: Exclude<ProviderHold, "spend_limit" | "retry_after">;
    };

/** A hold whose end the read gives. */
export type TimedHold = Extract<HoldFacts, { until: string }>;

/**
 * The facts of one hold of one provider, or null when the read holds no new
 * start for it, such as a refusal whose retry time is not after the read. A
 * spend limit and a refusal take their end from the observation of their
 * provider. The API sends that observation in each read that holds the
 * provider for either reason, so a read without it holds nothing for them.
 */
export type HoldFactsOf = (
  provider: OverflowProvider,
  hold: ProviderHold,
) => HoldFacts | null;

/** The facts of each provider hold of a live queue read at `readAt`. */
export function holdFacts(page: JobsLivePage, readAt: number): HoldFactsOf {
  return (provider, hold) => {
    switch (hold) {
      case "spend_limit": {
        const overflow = observation(page, provider);
        if (overflow === null) return null;
        return { provider, hold, until: overflow.next_month.slice(0, 10) };
      }
      case "retry_after": {
        const { retryUntil } = providerHold(page, provider, readAt);
        if (retryUntil === null) return null;
        return { provider, hold, until: retryUntil };
      }
      default:
        return { provider, hold };
    }
  };
}

/**
 * When a held provider takes jobs again, as a clause: "Boat takes jobs again
 * on 1 December", or "ciproxy tries Boat again at 10:13" in the time zone of
 * the browser.
 */
export function againText(facts: TimedHold): string {
  const name = providerName(facts.provider);
  switch (facts.hold) {
    case "spend_limit":
      return `${name} takes jobs again on ${formatUtcDate(facts.until)}`;
    case "retry_after":
      return `ciproxy tries ${name} again at ${formatTime(facts.until)}`;
  }
}
