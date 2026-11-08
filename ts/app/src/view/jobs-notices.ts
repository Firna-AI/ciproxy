// The notices of the live queue: each reason why jobs wait that the API
// reports. A notice names only the facts that the API gives, and each notice
// about a waiting label reads the same place of that label, so no notice
// promises a Boat start that the facts do not confirm. An action that changes
// a setting goes to a caller that the API says may manage it, and everyone
// else reads "Ask an Admin". A link to GitHub is for everyone, because GitHub
// decides who may act there.

import type { JobsLivePage } from "../api/jobs-types.ts";
import type { NoticeTone } from "./page-model.ts";
import { formatGigabytes } from "./format.ts";
import { boatNotices } from "./jobs-boat-notices.ts";
import { boatTiming, labelPlaces, type LabelPlace } from "./jobs-places.ts";

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

function jobs(count: number | bigint): string {
  return BigInt(count) === 1n ? "1 job" : `${count} jobs`;
}

/**
 * "when it has waited 30 seconds", "when they have waited 30 seconds", or "at
 * once": the saved overflow wait, for one job or for several.
 */
function waitText(seconds: number, several = false): string {
  if (seconds === 0) return "at once";
  const waited = several ? "when they have waited" : "when it has waited";
  if (seconds % 60 === 0) {
    const minutes = seconds / 60;
    return `${waited} ${minutes} ${minutes === 1 ? "minute" : "minutes"}`;
  }
  return `${waited} ${seconds} seconds`;
}

function sumJobs(places: readonly LabelPlace[]): number {
  return places.reduce((sum, place) => sum + place.jobs, 0);
}

function labelNotices(place: LabelPlace, settings: string): LiveNotice[] {
  const { fit } = place;
  const count = BigInt(fit.waiting_jobs);
  const verb = count === 1n ? "waits" : "wait";
  const action: NoticeAction | null = fit.can_edit
    ? { kind: "go", label: "Edit label", href: settings }
    : fit.ask_admin
      ? askAdmin
      : null;
  const size = fit.current?.label ?? null;
  const asked = size ? `${size.vcpus} vCPU and ${size.memory_gb} GB` : null;
  const notices: LiveNotice[] = [];
  if (fit.nothing_fits === true) {
    notices.push({
      key: `fits-${fit.label_id}`,
      tone: "warning",
      title: `${jobs(count)} with the label ${fit.label_name} ${verb}`,
      body: asked
        ? `No machine and no Boat size has ${asked}. Change the label, or let it run on the largest Boat size.`
        : "No machine and no Boat size fits this label.",
      action,
    });
  } else if (fit.nothing_fits === null || fit.nothing_fits === undefined) {
    notices.push({
      key: `fit-unknown-${fit.label_id}`,
      tone: "info",
      title: `${jobs(count)} with the label ${fit.label_name} ${verb}`,
      body: "Some machine or Boat facts are not available now, so ciproxy cannot tell where this label fits. The jobs start as soon as a machine or Boat can run them.",
      action: null,
    });
  }
  if (place.boat.kind === "starts" && place.boat.largest) {
    const boat = `Boat ${place.boat.sizeName}, with ${place.boat.size.vcpus} vCPU and ${formatGigabytes(place.boat.size.memory_bytes)}`;
    notices.push({
      key: `largest-${fit.label_id}`,
      tone: "info",
      title: `Jobs with the label ${fit.label_name} start on the largest Boat size`,
      body: asked
        ? `No machine and no Boat size has ${asked}, so ciproxy starts them on ${boat}.`
        : `ciproxy starts them on ${boat}.`,
      action,
    });
  }
  return notices;
}

/**
 * What Boat does for the jobs that wait for a free slot: it starts each one,
 * or some of them, after the overflow wait, or the Boat costs hold every
 * start. A label that Boat does not start now gets no Boat sentence.
 */
function slotBoat(
  page: JobsLivePage,
  forSlot: readonly LabelPlace[],
  slot: number,
): string[] {
  const afterWait = sumJobs(
    forSlot.filter((place) => boatTiming(place) === "after-wait"),
  );
  const wait = page.overflow_wait_seconds;
  if (afterWait === slot) return [`Each one starts on Boat ${waitText(wait)}.`];
  if (afterWait > 0) {
    const verb = afterWait === 1 ? "starts" : "start";
    return [
      `${afterWait} of them ${verb} on Boat ${waitText(wait, afterWait > 1)}.`,
    ];
  }
  if (forSlot.some((place) => place.boat.kind === "uncounted")) {
    return ["Boat starts no new sandbox until ciproxy can count its costs."];
  }
  return [];
}

/**
 * The notice of a full queue. It counts the jobs that wait for a free own
 * slot, and promises Boat only to the jobs of a label that Boat starts now:
 * after the overflow wait, or at once when no own machine runs the label.
 */
function fullNotice(
  page: JobsLivePage,
  places: readonly LabelPlace[],
  settings: string,
  boat: boolean,
): LiveNotice | null {
  const used = page.own_slots_used ?? null;
  const total = page.own_slots_total ?? null;
  if (used === null || total === null) return null;
  if (BigInt(total) === 0n || BigInt(used) < BigInt(total)) return null;
  const forSlot = places.filter((place) => place.own === "fits");
  const slot = sumJobs(forSlot);
  const atOnce = sumJobs(
    places.filter((place) => boatTiming(place) === "at-once"),
  );
  if (slot === 0 && atOnce === 0) return null;
  const manage = manager(page, settings);
  const lead = `${jobs(slot)} ${slot === 1 ? "waits" : "wait"} for a free slot.`;
  const notice = (
    sentences: readonly string[],
    action: NoticeAction | null,
  ): LiveNotice => ({
    key: "machines-full",
    tone: "info",
    title: "Your machines are full",
    body: sentences.join(" "),
    action,
  });
  if (!page.overflow_enabled) {
    return notice(
      [lead, "Boat overflow is off, so each one starts when a slot is free."],
      manage("Turn on Boat"),
    );
  }
  if (page.overflow && !page.overflow.connected) {
    return notice(
      [lead, "Boat is not connected, so each one starts when a slot is free."],
      manage("Connect Boat"),
    );
  }
  if (boat) return null;
  const sentences = slot > 0 ? [lead, ...slotBoat(page, forSlot, slot)] : [];
  if (atOnce > 0) {
    const verb = atOnce === 1 ? "starts" : "start";
    sentences.push(`${jobs(atOnce)} ${verb} on Boat at once.`);
  }
  return notice(sentences, null);
}

export function liveNotices(
  page: JobsLivePage,
  settingsHref: string,
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
  const places = labelPlaces(page);
  const overflow = page.overflow ?? null;
  const boat =
    overflow === null
      ? []
      : boatNotices(overflow, places, manager(page, settingsHref));
  notices.push(...boat);
  for (const place of places) {
    notices.push(...labelNotices(place, settingsHref));
  }
  const full = fullNotice(page, places, settingsHref, boat.length > 0);
  if (full) notices.push(full);
  return notices;
}
