// The notices of the live queue: each reason why jobs wait that the API
// reports. A notice names only the facts that the API gives, and each notice
// about a waiting label reads the same places of that label, so no notice
// promises a provider start that the facts do not confirm. A notice about a
// provider names that provider. An action that changes a setting goes to a
// caller that the API says may manage it, and everyone else reads "Ask an
// Admin". A link to GitHub is for everyone, because GitHub decides who may
// act there. A hold of a provider shows once: in one notice for every job,
// or, when each waiting label waits for that hold alone and none fits own
// machines, in the notice of each label.

import type { JobsLivePage } from "../api/jobs-types.ts";
import type { OverflowProvider, ProviderHold } from "../api/schema.ts";
import type { NoticeTone } from "./page-model.ts";
import { fullNotice } from "./jobs-full-notice.ts";
import { labelNotices, routedHold } from "./jobs-label-notices.ts";
import { labelPlaces, providerHold, type LabelPlace } from "./jobs-places.ts";
import { providerNotices, type HoldNotice } from "./jobs-provider-notices.ts";

export type NoticeAction =
  | { kind: "go"; label: string; href: string }
  | { kind: "leave"; label: string; href: string }
  | { kind: "ask"; text: string };

export type LiveNotice = {
  key: string;
  tone: NoticeTone;
  title: string;
  body: string;
  action: NoticeAction | null;
};

const askAdmin: NoticeAction = { kind: "ask", text: "Ask an Admin" };

/**
 * The action of a setting: its link for a caller who may manage it, and
 * "Ask an Admin" for everyone else.
 */
function manager(page: JobsLivePage, settings: string) {
  return (label: string): NoticeAction =>
    page.can_manage ? { kind: "go", label, href: settings } : askAdmin;
}

/**
 * The way forward of a label notice, as the API allows it: Edit label for a
 * caller who may edit the label, "Ask an Admin" when the API says so, or none.
 */
function recovery(place: LabelPlace, settings: string): NoticeAction | null {
  if (place.fit.can_edit) {
    return { kind: "go", label: "Edit label", href: settings };
  }
  return place.fit.ask_admin ? askAdmin : null;
}

/**
 * Whether the notices of the waiting labels state a hold in place of its
 * notice for every job: each waiting label waits for that hold alone, and no
 * waiting label fits own machines.
 */
function statedByLabels(
  places: readonly LabelPlace[],
  entry: HoldNotice,
): boolean {
  return (
    places.length > 0 &&
    places.every((place) => {
      const held = routedHold(place);
      return (
        place.own !== "fits" &&
        held?.provider === entry.provider &&
        held.hold === entry.hold
      );
    })
  );
}

/**
 * The notices of a live queue that the browser read at `readAt`. The read time
 * decides whether a saved retry time of a provider still holds new starts.
 */
export function liveNotices(
  page: JobsLivePage,
  settingsHref: string,
  readAt: number,
): LiveNotice[] {
  const notices: LiveNotice[] = page.visibility.map((item) =>
    item.suspended
      ? {
          key: `suspended-${item.organization}`,
          tone: "warning" as const,
          title: `The GitHub App is suspended on ${item.organization}`,
          body: `New jobs of ${item.organization} wait on GitHub until an owner unsuspends it, and running jobs finish.`,
          action: {
            kind: "leave",
            label: "Open GitHub",
            href: item.github_url,
          },
        }
      : {
          key: `removed-${item.organization}`,
          tone: "warning" as const,
          title: `The GitHub App was removed from ${item.organization}`,
          body: `Jobs of ${item.organization} no longer run on ciproxy. An owner can install the App again to keep its job history.`,
          action: {
            kind: "leave",
            label: "Install the App",
            href: item.github_url,
          },
        },
  );
  const places = labelPlaces(page, readAt);
  const manage = manager(page, settingsHref);
  const holds = page.overflow
    .flatMap((overflow) =>
      providerNotices(
        overflow,
        providerHold(page, overflow.provider, readAt),
        places,
        manage,
      ),
    )
    .filter((entry) => !statedByLabels(places, entry));
  notices.push(...holds.map((entry) => entry.notice));
  const shown = (provider: OverflowProvider, hold: ProviderHold) =>
    holds.some((entry) => entry.provider === provider && entry.hold === hold);
  for (const place of places) {
    notices.push(
      ...labelNotices(place, recovery(place, settingsHref), manage, shown),
    );
  }
  const othersStart = places.some((place) =>
    place.providers.some((item) => item.start.kind === "starts"),
  );
  const full = fullNotice(
    page,
    places,
    manage,
    holds.length > 0 && !othersStart,
  );
  if (full) notices.push(full);
  return notices;
}
