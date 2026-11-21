// The notices of a Boat hold: the spend limit, a rejected key, billing on the
// Boat account, and a refusal with a retry time that is after the read. While
// Boat takes no new job, a waiting job can start only on an own machine that
// runs its label. So a notice says that every waiting job waits for a free
// own slot only when each waiting label fits the own machines. Otherwise it
// says so only of the jobs whose label fits them, and it promises the other
// jobs no place. A refused Boat size holds only the labels that use it, so it
// raises no notice here.

import type { JobOverflow } from "../api/jobs-types.ts";
import { addUtcDays, formatCents, formatUtcDate } from "./exact-format.ts";
import { formatTime } from "./format.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { BoatHold, LabelPlace } from "./jobs-places.ts";

const monthName = new Intl.DateTimeFormat("en-GB", {
  month: "long",
  timeZone: "UTC",
});

/** Where the waiting jobs wait during a hold, for every job only when true. */
function holdWords(places: readonly LabelPlace[]) {
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

export function boatNotices(
  overflow: JobOverflow,
  hold: BoatHold,
  places: readonly LabelPlace[],
  manage: (label: string) => NoticeAction,
): LiveNotice[] {
  const words = holdWords(places);
  const notices: LiveNotice[] = [];
  if (overflow.limit_reached === true) {
    const until = overflow.next_month.slice(0, 10);
    const month = monthName.format(new Date(addUtcDays(until, -1)));
    const limit = overflow.limit_cents ?? null;
    notices.push({
      key: "spend-limit",
      tone: "warning",
      title: limit
        ? `Boat reached its ${formatCents(limit)} limit for ${month}`
        : `Boat reached its spend limit for ${month}`,
      body: `${words.wait}. Boat takes jobs again on ${formatUtcDate(until)}.`,
      action: manage("Raise the limit"),
    });
  }
  if (overflow.key_rejected) {
    notices.push({
      key: "boat-key",
      tone: "warning",
      title: "Boat rejected the key",
      body: `${words.wait}, until a new key is connected.`,
      action: manage("Connect a new key"),
    });
  }
  if (overflow.billing_required) {
    notices.push({
      key: "boat-billing",
      tone: "warning",
      title: "Boat needs billing on your Boat account",
      body: `Boat starts no new sandboxes until its billing is set up. ${words.wait}.`,
      action: null,
    });
  }
  if (hold.retryUntil !== null) {
    notices.push({
      key: "boat-refused",
      tone: "warning",
      title: "Boat refused new sandboxes",
      body: `${words.take}, and ciproxy tries Boat again at ${formatTime(hold.retryUntil)}.`,
      action: null,
    });
  }
  return notices;
}
