// The notices of one waiting label: no place of its order fits it, its fit
// is unknown, it starts on the largest size of a provider, a provider
// refused its size, or a place that fits it cannot take jobs now. A fit
// notice names the full request with "fits", and the places of the order. It
// never names the resource that is short, because the API does not say which
// one is. It offers the largest size only when the order holds a provider
// and the label has not chosen that size yet. A refused size holds only this
// label, so its notice speaks of this label only. When the routing of the
// API says that the label waits for a place that fits it, the notice gives
// the reason of that place, as the label list does, and not that nothing
// fits. It also gives when that place takes jobs again, from the facts of
// the hold at the read, as the notice of the provider does. A refusal whose
// retry time is not after the read holds nothing, so the notice leaves it
// out.

import type {
  LabelInput,
  OverflowProvider,
  ProviderHold,
} from "../api/schema.ts";
import { formatGigabytes } from "./format.ts";
import {
  againText,
  type HoldFacts,
  type HoldFactsOf,
} from "./jobs-hold-facts.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { LabelPlace, ProviderPlace } from "./jobs-places.ts";
import {
  capitalized,
  holdText,
  providerName,
  providerNames,
} from "./providers.ts";

/** "1 job", "3 jobs". */
export function jobsText(count: number | bigint): string {
  return BigInt(count) === 1n ? "1 job" : `${count} jobs`;
}

/** "8 vCPU, 16 GB of memory, and 60 GB of free disk": the full request. */
function requestText(label: LabelInput): string {
  return `${label.vcpus} vCPU, ${label.memory_gb} GB of memory, and ${label.disk_gb} GB of free disk`;
}

function names(items: readonly ProviderPlace[], join: "and" | "or" = "or") {
  return providerNames(
    items.map((item) => item.provider),
    join,
  );
}

/** Why the own machines run no job of the label, or null when unknown. */
function ownClause(place: LabelPlace, request: string): string | null {
  switch (place.own) {
    case "forbidden":
      return "the label does not run on your machines";
    case "no-machine":
      return "none of your machines takes jobs now";
    case "no-fit":
      return `no machine of yours fits a job of ${request}`;
    default:
      return null;
  }
}

/** Why no provider of the order runs the label, or null when not known. */
function providerClause(place: LabelPlace, request: string): string | null {
  const { providers } = place;
  if (providers.length === 0) return "the label runs on your machines only";
  if (providers.every((item) => item.fit.kind === "no-fit")) {
    return `no ${names(providers)} size fits a job of ${request}`;
  }
  return null;
}

/** Why no place of the order fits the label, from the facts of each place. */
function noPlaceText(place: LabelPlace, request: string): string {
  const { providers } = place;
  if (place.own === "forbidden" && providers.length === 0) {
    return "The order of the label has no place";
  }
  if (
    place.own === "no-fit" &&
    providers.length > 0 &&
    providers.every((item) => item.fit.kind === "no-fit")
  ) {
    return `No machine of yours and no ${names(providers)} size fits a job of ${request}`;
  }
  const own = ownClause(place, request);
  const other = providerClause(place, request);
  if (own === null || other === null) {
    return "No place of its order fits this label";
  }
  return `${capitalized(own)}, and ${other}`;
}

/** Why the jobs of the label start on the largest size of a provider. */
function largestText(
  place: LabelPlace,
  provider: OverflowProvider,
  request: string,
): string {
  const tried = place.providers
    .filter(
      (item) =>
        item.fit.kind === "no-fit" ||
        (item.fit.kind === "fits" && item.fit.largest),
    )
    .map((item) => item.provider);
  const named = providerNames(tried.length > 0 ? tried : [provider]);
  const sizes = `no ${named} size fits a job of ${request}`;
  if (place.own === "no-fit") {
    return `No machine of yours and ${sizes}`;
  }
  const own = ownClause(place, request);
  return own === null
    ? capitalized(sizes)
    : `${capitalized(own)}, and ${sizes}`;
}

/** The action that clears a hold, for a person who may change it. */
function holdAction(
  hold: ProviderHold,
  manage: (label: string) => NoticeAction,
): NoticeAction | null {
  switch (hold) {
    case "off":
      return manage("Turn on overflow");
    case "not_connected":
      return manage("Connect a provider");
    case "spend_limit":
      return manage("Raise the limit");
    case "key_rejected":
      return manage("Connect a new key");
    default:
      return null;
  }
}

/**
 * The provider hold that the notice of a waiting label states: the hold that
 * the routing of the API gives, with its facts at the read. A refused size
 * has a notice of its own, and the facts drop a refusal whose retry time is
 * not after the read.
 */
export function statedHold(
  place: LabelPlace,
  facts: HoldFactsOf,
): HoldFacts | null {
  const routing = place.routing;
  if (routing?.kind !== "waits" || routing.reason.kind !== "provider") {
    return null;
  }
  const { provider, hold } = routing.reason;
  return hold === "size_refused" ? null : facts(provider, hold);
}

export function labelNotices(
  place: LabelPlace,
  action: NoticeAction | null,
  manage: (label: string) => NoticeAction,
  /** The facts of each provider hold at the read. */
  facts: HoldFactsOf,
  /** A notice of this provider and hold shows already, for every job. */
  shown: (provider: OverflowProvider, hold: ProviderHold) => boolean,
): LiveNotice[] {
  const { fit } = place;
  const count = BigInt(fit.waiting_jobs);
  const verb = count === 1n ? "waits" : "wait";
  const title = `${jobsText(count)} with the label ${fit.label_name} ${verb}`;
  const label = fit.current?.label ?? null;
  const request = label ? requestText(label) : null;
  const notices: LiveNotice[] = [];
  const held = statedHold(place, facts);
  if (held !== null) {
    if (!shown(held.provider, held.hold)) {
      // A hold whose end the read gives says when the place takes jobs again.
      const again = "until" in held ? ` ${againText(held)}.` : "";
      notices.push({
        key: `held-${fit.label_id}`,
        tone: "warning",
        title,
        body: `${capitalized(holdText(held.provider, held.hold))}. No other place of its order can take them now.${again}`,
        action: holdAction(held.hold, manage),
      });
    }
  } else if (fit.nothing_fits === true) {
    const largestOpen =
      place.providers.some((item) => item.fit.kind === "no-fit") &&
      label?.when_nothing_fits === "wait";
    notices.push({
      key: `fits-${fit.label_id}`,
      tone: "warning",
      title,
      body: request
        ? `${noPlaceText(place, request)}. ${largestOpen ? "Change the label, or let it run on the largest size." : "Change the label."}`
        : "No place of its order fits this label.",
      action,
    });
  } else if (fit.nothing_fits === null || fit.nothing_fits === undefined) {
    const providers =
      place.providers.length > 0 ? names(place.providers) : null;
    const facts = providers ? `machine or ${providers}` : "machine";
    const runner = providers ? `a machine or ${providers}` : "a machine";
    notices.push({
      key: `fit-unknown-${fit.label_id}`,
      tone: "info",
      title,
      body: `Some ${facts} facts are not available now, so ciproxy cannot tell where this label fits. The jobs start as soon as ${runner} can run them.`,
      action: null,
    });
  }
  for (const item of place.providers) {
    const name = providerName(item.provider);
    if (item.start.kind === "starts" && item.start.largest) {
      const size = `${name} ${item.start.sizeName}, with ${item.start.size.vcpus} vCPU and ${formatGigabytes(item.start.size.memory_bytes)}`;
      notices.push({
        key: `largest-${item.provider}-${fit.label_id}`,
        tone: "info",
        title: `Jobs with the label ${fit.label_name} start on the largest ${name} size`,
        body: request
          ? `${largestText(place, item.provider, request)}, so ciproxy starts them on ${size}.`
          : `ciproxy starts them on ${size}.`,
        action,
      });
    }
    if (item.start.kind === "size-refused") {
      notices.push({
        key: `size-refused-${item.provider}-${fit.label_id}`,
        tone: "warning",
        title: `${name} refused the size of the label ${fit.label_name}`,
        body:
          place.own === "fits"
            ? `Its jobs take the first free slot on your machines, and ciproxy tries ${name} again.`
            : `Its jobs wait, and ciproxy tries ${name} again.`,
        action: null,
      });
    }
  }
  return notices;
}
