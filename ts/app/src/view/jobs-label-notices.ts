// The notices of one waiting label: no permitted place fits it, its fit is
// unknown, it starts on the largest Boat size, or Boat refused its size. A fit
// notice names the full request with "fits", and the places that the label
// forbids or that take no jobs. It never names the resource that is short,
// because the API does not say which one is. It offers the largest Boat size
// only when the label allows Boat and has not chosen that size yet. A refused
// size holds only this label, so its notice speaks of this label only.

import type { LabelInput } from "../api/schema.ts";
import { formatGigabytes } from "./format.ts";
import type { LiveNotice, NoticeAction } from "./jobs-notices.ts";
import type { LabelPlace } from "./jobs-places.ts";

/** "1 job", "3 jobs". */
export function jobsText(count: number | bigint): string {
  return BigInt(count) === 1n ? "1 job" : `${count} jobs`;
}

/** "8 vCPU, 16 GB of memory, and 60 GB of free disk": the full request. */
function requestText(label: LabelInput): string {
  return `${label.vcpus} vCPU, ${label.memory_gb} GB of memory, and ${label.disk_gb} GB of free disk`;
}

function sentence(clause: string): string {
  return clause.charAt(0).toUpperCase() + clause.slice(1);
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

/** Why no permitted place fits the label, from the facts of each place. */
function noPlaceText(place: LabelPlace, request: string): string {
  const boat = place.boatFit.kind;
  if (place.own === "no-fit" && boat === "no-fit") {
    return `No machine of yours and no Boat size fits a job of ${request}`;
  }
  if (place.own === "forbidden" && boat === "forbidden") {
    return "The label runs neither on your machines nor on Boat";
  }
  const own = ownClause(place, request);
  const boatText =
    boat === "forbidden"
      ? "the label does not run on Boat"
      : boat === "no-fit"
        ? `no Boat size fits a job of ${request}`
        : null;
  if (own === null || boatText === null) {
    return "No permitted machine or Boat size fits this label";
  }
  return `${sentence(own)}, and ${boatText}`;
}

/** Why the jobs of the label start on the largest Boat size. */
function largestText(place: LabelPlace, request: string): string {
  if (place.own === "no-fit") {
    return `No machine of yours and no Boat size fits a job of ${request}`;
  }
  const own = ownClause(place, request);
  const boat = `no Boat size fits a job of ${request}`;
  return own === null ? sentence(boat) : `${sentence(own)}, and ${boat}`;
}

export function labelNotices(
  place: LabelPlace,
  action: NoticeAction | null,
): LiveNotice[] {
  const { fit } = place;
  const count = BigInt(fit.waiting_jobs);
  const verb = count === 1n ? "waits" : "wait";
  const label = fit.current?.label ?? null;
  const request = label ? requestText(label) : null;
  const notices: LiveNotice[] = [];
  if (fit.nothing_fits === true) {
    const largestOpen =
      place.boatFit.kind === "no-fit" && label?.when_nothing_fits === "wait";
    notices.push({
      key: `fits-${fit.label_id}`,
      tone: "warning",
      title: `${jobsText(count)} with the label ${fit.label_name} ${verb}`,
      body: request
        ? `${noPlaceText(place, request)}. ${largestOpen ? "Change the label, or let it run on the largest Boat size." : "Change the label."}`
        : "No permitted machine or Boat size fits this label.",
      action,
    });
  } else if (fit.nothing_fits === null || fit.nothing_fits === undefined) {
    notices.push({
      key: `fit-unknown-${fit.label_id}`,
      tone: "info",
      title: `${jobsText(count)} with the label ${fit.label_name} ${verb}`,
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
      body: request
        ? `${largestText(place, request)}, so ciproxy starts them on ${boat}.`
        : `ciproxy starts them on ${boat}.`,
      action,
    });
  }
  if (place.boat.kind === "size-refused") {
    notices.push({
      key: `size-refused-${fit.label_id}`,
      tone: "warning",
      title: `Boat refused the size of the label ${fit.label_name}`,
      body:
        place.own === "fits"
          ? "Its jobs take the first free slot on your machines, and ciproxy tries Boat again."
          : "Its jobs wait, and ciproxy tries Boat again.",
      action: null,
    });
  }
  return notices;
}
