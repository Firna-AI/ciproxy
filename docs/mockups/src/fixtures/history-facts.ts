// Finished jobs for the History states that come from the current API: a
// filtered list, a job that no ciproxy runner took, and a job whose finish
// time GitHub did not give. The API gives no total of the day and no tier
// name, so these states count the jobs that the list shows and name the
// history window of the plan in days.

import { history } from "./job-history.ts";
import { none, own, type Job } from "./jobs.ts";

/** The history window of Team, in days, as the API gives it. */
export const historyDays = 30;

/** The date choices that the window of Team allows. */
export const dateChoices = ["Today", "Last 7 days", "Last 30 days"] as const;

/** The search and the place of the filtered state. */
export const historyFilter = { search: "e2e", place: "boat" } as const;

/** Finished jobs that ran on Boat and match "e2e". */
export const filteredHistory: Job[] = history.filter(
  (job) =>
    job.where.kind === historyFilter.place &&
    job.name.includes(historyFilter.search),
);

/** A job of the release that was cancelled before a runner took it. */
export const noRunnerJob: Job = {
  id: "facts-no-runner",
  name: "e2e (webkit) 2/2",
  workflow: "E2E",
  repo: "mokly",
  ref: "#1537",
  state: "cancelled",
  where: none,
  time: "Unknown",
  label: "ciproxy",
  waited: "Unknown",
  finished: "14:21",
};

/** A job that GitHub reports as finished, with no finish time. */
export const untimedJob: Job = {
  id: "facts-untimed",
  name: "deploy preview",
  workflow: "Preview",
  repo: "mokly-cloud",
  ref: "#320",
  state: "passed",
  where: own(3, 1),
  time: "Unknown",
  label: "ciproxy",
  waited: "0s",
  finished: "Unknown",
};

/** The history as the API gives it, with the two jobs of unknown facts. */
export const currentHistory: Job[] = [
  ...history.slice(0, 2),
  untimedJob,
  ...history.slice(2, 8),
  noRunnerJob,
  ...history.slice(8),
];
