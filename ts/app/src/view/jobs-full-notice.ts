// The notice of a full queue: the jobs that wait for a free slot on own
// machines, and which provider starts them, and when. A job goes to the
// first place of its label order that can take it: a provider before own
// machines starts it at once, as does a provider after own machines that no
// own machine fits, and a provider after own machines that fit starts it
// after the overflow wait. A fact that lacks before that provider promises
// nothing. The switch that holds every provider, and a workspace with no
// connected provider, name no provider.

import type { JobsLivePage } from "../api/jobs-types.ts";
import type { OverflowProvider } from "../api/schema.ts";
import { jobsText } from "./jobs-label-notices.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { LabelPlace, OwnFit } from "./jobs-places.ts";
import { providerName } from "./providers.ts";

/** No own machine runs the label: it forbids them, or none takes or fits it. */
export function noOwnMachine(own: OwnFit): boolean {
  return own === "forbidden" || own === "no-machine" || own === "no-fit";
}

/** A provider that starts the jobs of a label now, and when. */
export type ProviderTiming = {
  provider: OverflowProvider;
  timing: "at-once" | "after-wait";
};

/**
 * Which provider starts the jobs of a label, and when: the first provider of
 * the order that starts them now. It starts them at once when it comes
 * before own machines or when no own machine runs the label, and after the
 * overflow wait when own machines that fit come first. A fact that lacks
 * before that provider promises nothing.
 */
export function providerTiming(place: LabelPlace): ProviderTiming | null {
  for (const item of place.providers) {
    if (item.start.kind === "starts") {
      if (item.beforeOwn || noOwnMachine(place.own)) {
        return { provider: item.provider, timing: "at-once" };
      }
      return place.own === "fits"
        ? { provider: item.provider, timing: "after-wait" }
        : null;
    }
    if (item.start.kind === "unknown") return null;
  }
  return null;
}

/**
 * The first provider after own machines that would take the jobs of a label
 * that fits them, but whose costs have no safe count now.
 */
export function uncountedNext(place: LabelPlace): OverflowProvider | null {
  if (place.own !== "fits") return null;
  for (const item of place.providers) {
    if (item.beforeOwn) continue;
    if (item.start.kind === "uncounted") return item.provider;
    if (item.start.kind === "starts" || item.start.kind === "unknown") {
      return null;
    }
  }
  return null;
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

/** The jobs of some labels for each provider that starts them, in order. */
function byProvider(
  places: readonly LabelPlace[],
  pick: (place: LabelPlace) => OverflowProvider | null,
): { provider: OverflowProvider; jobs: number }[] {
  const counts = new Map<OverflowProvider, number>();
  for (const place of places) {
    const provider = pick(place);
    if (provider !== null) {
      counts.set(provider, (counts.get(provider) ?? 0) + place.jobs);
    }
  }
  return [...counts].map(([provider, jobs]) => ({ provider, jobs }));
}

/**
 * What the providers do for the jobs that wait for a free slot: a provider
 * starts each one, or some of them, after the overflow wait, or the costs of
 * the next provider hold every start. A label that no provider starts now
 * gets no provider sentence.
 */
function slotSentences(
  page: JobsLivePage,
  forSlot: readonly LabelPlace[],
  slot: number,
): string[] {
  const wait = page.overflow_wait_seconds;
  const afterWait = byProvider(forSlot, (place) => {
    const timing = providerTiming(place);
    return timing?.timing === "after-wait" ? timing.provider : null;
  });
  const [only] = afterWait;
  if (afterWait.length === 1 && only !== undefined && only.jobs === slot) {
    return [
      `Each one starts on ${providerName(only.provider)} ${waitText(wait)}.`,
    ];
  }
  if (afterWait.length > 0) {
    return afterWait.map(({ provider, jobs }) => {
      const verb = jobs === 1 ? "starts" : "start";
      return `${jobs} of them ${verb} on ${providerName(provider)} ${waitText(wait, jobs > 1)}.`;
    });
  }
  return byProvider(forSlot, uncountedNext).map(
    ({ provider }) =>
      `${providerName(provider)} starts no new sandbox until ciproxy can count its costs.`,
  );
}

/**
 * The notice of a full queue. It counts the jobs that wait for a free own
 * slot, and promises a provider only to the jobs of a label that the
 * provider starts now: after the overflow wait, or at once when it comes
 * before own machines or no own machine runs the label.
 */
export function fullNotice(
  page: JobsLivePage,
  places: readonly LabelPlace[],
  manage: (label: string) => NoticeAction,
  held: boolean,
): LiveNotice | null {
  const used = page.own_slots_used ?? null;
  const total = page.own_slots_total ?? null;
  if (used === null || total === null) return null;
  if (BigInt(total) === 0n || BigInt(used) < BigInt(total)) return null;
  const atOnce = byProvider(places, (place) => {
    const timing = providerTiming(place);
    return timing?.timing === "at-once" ? timing.provider : null;
  });
  const forSlot = places.filter(
    (place) =>
      place.own === "fits" && providerTiming(place)?.timing !== "at-once",
  );
  const slot = sumJobs(forSlot);
  if (slot === 0 && atOnce.length === 0) return null;
  const lead = `${jobsText(slot)} ${slot === 1 ? "waits" : "wait"} for a free slot.`;
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
      [
        lead,
        "Overflow providers are off, so each one starts when a slot is free.",
      ],
      manage("Turn on overflow"),
    );
  }
  if (page.overflow.length > 0 && page.overflow.every((o) => !o.connected)) {
    return notice(
      [
        lead,
        "No overflow provider is connected, so each one starts when a slot is free.",
      ],
      manage("Connect a provider"),
    );
  }
  if (held) return null;
  const sentences =
    slot > 0 ? [lead, ...slotSentences(page, forSlot, slot)] : [];
  for (const { provider, jobs } of atOnce) {
    const verb = jobs === 1 ? "starts" : "start";
    sentences.push(
      `${jobsText(jobs)} ${verb} on ${providerName(provider)} at once.`,
    );
  }
  return notice(sentences, null);
}
