// The job page as a model: where the job ran and why, the times that GitHub
// gave, and the details of its label, its VM, its wait, its cache, and its
// start. A request that the API did not keep says so and never takes the
// current size of the label. A job that no runner took names no VM, no wait,
// no cache, and no start, because it is not a start.

import type { JobView } from "../api/jobs-types.ts";
import { formatCount } from "./exact-format.ts";
import {
  boatText,
  durationText,
  jobLook,
  placeOf,
  requestText,
  subtitleText,
  unknownText,
  vmText,
  type Place,
  type ResultLook,
} from "./jobs-format.ts";

export type DetailRowModel = {
  label: string;
  value: string;
  caption: string | null;
  /** The value is a code name, such as a label. */
  code?: true;
};

export type TimelineEvent = {
  time: string;
  text: string;
  machine: string | null;
};

export type JobDetailModel = {
  title: string;
  look: ResultLook;
  subtitle: string;
  githubUrl: string | null;
  place: Place;
  /** The place as a title, such as "forge-01 · slot 3", "Boat", or "No runner". */
  placeTitle: string;
  reason: string;
  timeline: readonly TimelineEvent[];
  rows: readonly DetailRowModel[];
};

export const unavailableReason = "Why the job ran here is not available.";

export const noRunnerReason =
  "No ciproxy runner took this job, so it is not a start.";

const cacheSources = {
  machine: "This machine",
  github: "The GitHub cache",
  machine_and_github: "This machine and the GitHub cache",
} as const;

function placeTitle(place: Place): string {
  switch (place.kind) {
    case "machine":
      return place.slot === null ? place.name : `${place.name} · ${place.slot}`;
    case "boat":
      return "Boat";
    case "none":
      return "No runner";
    case "unknown":
      return unknownText;
  }
}

function vmRow(job: JobView, place: Place): DetailRowModel {
  const request = requestText(job.label);
  const actual = job.actual_size ?? null;
  if (job.location?.kind === "boat") {
    return {
      label: "VM size",
      value: boatText(job.location.size, actual),
      caption: "Boat bills this size.",
    };
  }
  if (place.kind === "machine" && request !== null) {
    return { label: "VM size", value: request, caption: null };
  }
  return {
    label: "VM size",
    value: actual === null ? unknownText : vmText(actual),
    caption: null,
  };
}

/** "2.1 GB": restored bytes in decimal gigabytes, to one place. */
function restoredSize(bytes: string): string {
  return `${(Number(bytes) / 1e9).toLocaleString("en-US", { maximumFractionDigits: 1 })} GB`;
}

function cacheRows(job: JobView, place: Place): DetailRowModel[] {
  const counts = job.cache_counts ?? null;
  const source = job.cache_source ?? null;
  if (counts !== null && /^0+$/.test(counts.restored_entries)) {
    return [{ label: "Cache", value: "No entries restored", caption: null }];
  }
  const rows: DetailRowModel[] = [];
  if (counts !== null) {
    rows.push({
      label: "Cache",
      value: `${formatCount(counts.restored_entries)} restored · ${restoredSize(counts.restored_bytes)}`,
      caption: null,
    });
  }
  if (source !== null) {
    const fellBack =
      counts === null && source === "github" && place.kind === "machine";
    rows.push({
      label: "Cache source",
      value: cacheSources[source],
      caption: fellBack
        ? `The cache of ${place.name} was not available, so the job used the GitHub cache.`
        : null,
    });
  }
  if (rows.length === 0) {
    rows.push({ label: "Cache", value: unknownText, caption: null });
  }
  return rows;
}

function labelRow(job: JobView): DetailRowModel {
  const request = requestText(job.label);
  return {
    label: "Label",
    value: job.label.name,
    code: true,
    caption:
      request === null
        ? "Requested size unknown"
        : job.location?.kind === "boat"
          ? request
          : null,
  };
}

function timeOf(iso: string, timeZone?: string): string {
  return new Intl.DateTimeFormat("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    timeZone,
  }).format(new Date(iso));
}

function timeline(job: JobView, place: Place, timeZone?: string) {
  const events: TimelineEvent[] = [];
  const add = (
    iso: string | null | undefined,
    text: string,
    machine: string | null = null,
  ) => {
    if (iso !== null && iso !== undefined) {
      events.push({ time: timeOf(iso, timeZone), text, machine });
    }
  };
  add(job.queued_at, "Queued on GitHub");
  if (place.kind === "machine")
    add(job.started_at, "Taken by a runner on", place.name);
  else if (place.kind === "boat")
    add(job.started_at, "Taken by a runner on Boat");
  else add(job.started_at, "Taken by a runner");
  if (job.status === "completed") add(job.completed_at, jobLook(job).label);
  return events;
}

export function jobDetailModel(
  job: JobView,
  timeZone?: string,
): JobDetailModel {
  const place = placeOf(job.location, job.counts_as_start);
  const ran = place.kind !== "none";
  const rows: DetailRowModel[] = [labelRow(job)];
  if (ran) {
    rows.push(vmRow(job, place));
    rows.push({
      label: "Waited",
      value: durationText(job.waited_seconds),
      caption: null,
    });
    rows.push({
      label: "Duration",
      value: durationText(job.runtime_seconds),
      caption: null,
    });
    rows.push(...cacheRows(job, place));
    if (job.counts_as_start) {
      rows.push({ label: "Starts", value: "Counts as 1 start", caption: null });
    }
  }
  return {
    title: job.name,
    look: jobLook(job),
    subtitle: subtitleText(job),
    githubUrl: job.github_url ?? null,
    place,
    placeTitle: placeTitle(place),
    reason: ran ? (job.routing_reason ?? unavailableReason) : noRunnerReason,
    timeline: timeline(job, place, timeZone),
    rows,
  };
}
