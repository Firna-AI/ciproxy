// The live queue as a model: the figures, the waiting and running rows, and
// the quiet state. A figure that the API leaves out says "Unknown", and the
// waiting figure leaves the row while ciproxy cannot see every waiting job.
// A waiting job shows its label, not a machine, because GitHub gives it to
// whichever runner is idle first.

import type { JobView, JobsLivePage } from "../api/jobs-types.ts";
import type { Figure } from "../components/figures.tsx";
import { formatCount } from "./exact-format.ts";
import {
  contextText,
  durationText,
  placeOf,
  unknownText,
  waitingText,
  type Place,
} from "./jobs-format.ts";

export type LiveRow = {
  id: string;
  name: string;
  context: string;
  /** The label of a waiting job. */
  label: string;
  /** Where a running job runs. */
  place: Place;
  /** Waiting for, or running for. */
  time: string | null;
};

export type LiveModel = {
  figures: readonly Figure[];
  waiting: readonly LiveRow[];
  running: readonly LiveRow[];
  /** The block that shows when no job runs or waits, else null. */
  quiet: { title: string; warm: boolean } | null;
};

/** "3 of 18" when both slot figures are known, else "Unknown". */
function slotsText(page: JobsLivePage): string {
  const used = page.own_slots_used ?? null;
  const total = page.own_slots_total ?? null;
  if (used === null || total === null) return unknownText;
  return `${formatCount(used)} of ${formatCount(total)}`;
}

function waitingRow(job: JobView, readAt: number): LiveRow {
  return {
    id: job.id,
    name: job.name,
    context: contextText(job),
    label: job.label.name,
    place: { kind: "unknown" },
    time: waitingText(job.queued_at, readAt),
  };
}

function runningRow(job: JobView): LiveRow {
  return {
    id: job.id,
    name: job.name,
    context: contextText(job),
    label: job.label.name,
    place: placeOf(job.location, job.counts_as_start),
    time: durationText(job.runtime_seconds),
  };
}

export function liveModel(page: JobsLivePage, readAt: number): LiveModel {
  const waitingCount = page.waiting_count ?? null;
  const figures: Figure[] = [
    { label: "Running", value: formatCount(BigInt(page.running.length)) },
  ];
  if (waitingCount !== null) {
    figures.push({ label: "Waiting", value: formatCount(waitingCount) });
  }
  figures.push(
    { label: "Own slots in use", value: slotsText(page) },
    { label: "On Boat", value: formatCount(page.on_boat) },
  );
  const warm = BigInt(page.warm_runners);
  const idle = page.running.length === 0 && page.waiting.length === 0;
  return {
    figures,
    waiting: page.waiting.map((job) => waitingRow(job, readAt)),
    running: page.running.map(runningRow),
    quiet: idle
      ? {
          title:
            warm === 0n
              ? "No job runs now"
              : `${formatCount(warm)} warm ${warm === 1n ? "runner is" : "runners are"} ready`,
          warm: warm > 0n,
        }
      : null,
  };
}
