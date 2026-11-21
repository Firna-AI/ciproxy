// Finished jobs with their result, where each ran, and how long each waited.
// The filters stay above the list in every state. The known-data state counts
// the jobs of the day from the story. The states of the current API count
// the jobs that the list shows, and name the history window in days, because
// the API gives no total and no tier name.

import { ListChecks, SearchX } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { LinkTabs, PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { formatCount } from "../../fixtures/format.ts";
import {
  currentHistory,
  filteredHistory,
  historyDays,
  historyFilter,
  noRunnerJob,
  untimedJob,
} from "../../fixtures/history-facts.ts";
import { history, jobsToday } from "../../fixtures/job-history.ts";
import type { Job } from "../../fixtures/jobs.ts";
import { tiers } from "../../fixtures/tiers.ts";
import {
  HistoryFilters,
  type HistoryFilterValues,
} from "./history-filters.tsx";
import { jobTabs } from "./jobs-live.tsx";
import { JobList, type RowLinks } from "./job-tables.tsx";
import "./jobs.css";

export const historyStates = [
  "history",
  "empty",
  "no-match",
  "date-menu",
  "filtered",
  "current",
] as const;
export type HistoryState = (typeof historyStates)[number];

const allToday: HistoryFilterValues = {
  search: "",
  place: "all",
  date: "Today",
};

const filters: Readonly<Record<HistoryState, HistoryFilterValues>> = {
  history: allToday,
  empty: allToday,
  "no-match": { search: "deploy", place: "boat", date: "Today" },
  "date-menu": { ...allToday, menu: true },
  filtered: { ...historyFilter, date: "Today" },
  current: allToday,
};

const lists: Readonly<Record<HistoryState, readonly Job[]>> = {
  history,
  empty: [],
  "no-match": [],
  "date-menu": history,
  filtered: filteredHistory,
  current: currentHistory,
};

/** In the current-API state, rows open the pages of the current API. */
const currentLinks: RowLinks = {
  [history[0]?.id ?? ""]: paths.jobCurrentOwn,
  [history[1]?.id ?? ""]: paths.jobCurrentBoat,
  [untimedJob.id]: paths.jobUnknownFacts,
  [noRunnerJob.id]: paths.jobNoRunner,
};

/** "11 jobs today · Your plan keeps 30 days of history". */
function currentFooter(count: number): string {
  const jobs = count === 1 ? "1 job" : `${formatCount(count)} jobs`;
  return `${jobs} today · Your plan keeps ${historyDays} days of history`;
}

function footer(state: HistoryState, count: number): string {
  if (state === "history" || state === "date-menu") {
    return `${count} of ${formatCount(jobsToday)} jobs today · ${tiers.team.name} keeps ${tiers.team.history} of history`;
  }
  return currentFooter(count);
}

function EmptyList({ state }: { state: HistoryState }) {
  if (state === "no-match") {
    return (
      <StateBlock
        icon={SearchX}
        title="No finished jobs match"
        action={
          <LinkAction to={paths.jobsHistory} tone="primary">
            Clear filters
          </LinkAction>
        }
      >
        Try another search, place, or date.
      </StateBlock>
    );
  }
  return (
    <StateBlock icon={ListChecks} title="Finished jobs show here">
      Each job with <span className="inline-code">runs-on: ciproxy</span> shows
      its result, where it ran, and how long it waited.
    </StateBlock>
  );
}

export function JobsHistoryScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: HistoryState;
}) {
  const jobs = lists[state];
  return (
    <AppShell
      viewport={viewport}
      tier={state === "empty" ? "free" : "team"}
      active="jobs"
      setup={
        state === "empty" ? { done: 3, total: 4, active: false } : undefined
      }
    >
      <PageHeader title="Jobs" />
      <LinkTabs tabs={jobTabs} active={paths.jobsHistory} />
      <HistoryFilters values={filters[state]} />
      {jobs.length === 0 ? (
        <Panel>
          <EmptyList state={state} />
        </Panel>
      ) : (
        <Panel flush>
          <JobList
            viewport={viewport}
            kind="finished"
            jobs={jobs}
            links={state === "current" ? currentLinks : undefined}
          />
          <p className="more-row caption">{footer(state, jobs.length)}</p>
        </Panel>
      )}
    </AppShell>
  );
}
