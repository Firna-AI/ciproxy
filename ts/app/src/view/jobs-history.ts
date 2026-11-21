// The History list as a model: its footer and its empty state. The API gives
// no total of the day and no tier name, so the footer counts the jobs that
// the list shows and names the history window of the plan in days.

import type { JobView } from "../api/jobs-types.ts";
import { formatCount } from "./exact-format.ts";
import { countText, formatTime } from "./format.ts";
import {
  contextText,
  durationText,
  jobLook,
  placeOf,
  unknownText,
  type Place,
  type ResultLook,
} from "./jobs-format.ts";
import { filtered, jobHref, type HistoryFilters } from "./jobs-links.ts";

/** "11 jobs today · Your plan keeps 30 days of history". */
export function historyFooter(
  count: number,
  filters: HistoryFilters,
  historyDays: number,
): string {
  const shown = `${formatCount(BigInt(count))} ${count === 1 ? "job" : "jobs"}`;
  const when =
    filters.days === 1 ? "today" : `in the last ${filters.days} days`;
  const keeps = countText(historyDays, "day", "days");
  return `${shown} ${when} · Your plan keeps ${keeps} of history`;
}

export type HistoryEmpty = { title: string; body: string; clear: boolean };

/**
 * The empty list. With a filter set, it says that no job matches and offers
 * to clear the filters. Without one, it says what the list shows.
 */
export function historyEmpty(filters: HistoryFilters): HistoryEmpty {
  if (filtered(filters)) {
    return {
      title: "No finished jobs match",
      body: "Try another search, place, or date.",
      clear: true,
    };
  }
  return {
    title: "Finished jobs show here",
    body: "Each job with runs-on: ciproxy shows its result, where it ran, and how long it waited.",
    clear: false,
  };
}

export type FinishedRow = {
  id: string;
  look: ResultLook;
  name: string;
  href: string;
  context: string;
  place: Place;
  waited: string;
  duration: string;
  /** The clock time of the finish, or "Unknown" when GitHub gave none. */
  finished: string;
};

/** One finished job as a row of History. */
export function finishedRow(
  job: JobView,
  workspaceId: string,
  timeZone?: string,
): FinishedRow {
  const done = job.completed_at ?? null;
  return {
    id: job.id,
    look: jobLook(job),
    name: job.name,
    href: jobHref(workspaceId, job.id),
    context: contextText(job),
    place: placeOf(job.location, job.counts_as_start),
    waited: durationText(job.waited_seconds),
    duration: durationText(job.runtime_seconds),
    finished: done === null ? unknownText : formatTime(done, timeZone),
  };
}
