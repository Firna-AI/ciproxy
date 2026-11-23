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
import {
  history,
  jobsToday,
  twoProviderHistory,
} from "../../fixtures/job-history.ts";
import type { Job } from "../../fixtures/jobs.ts";
import type { ProviderKey } from "../../fixtures/providers.ts";
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
  "two-providers",
] as const;
export type HistoryState = (typeof historyStates)[number];

/** The providers that ran jobs on 18 November in the main story. */
const boatOnly: readonly ProviderKey[] = ["boat"];

const allToday: HistoryFilterValues = {
  search: "",
  place: "all",
  providers: boatOnly,
  date: "Today",
};

const filters: Readonly<Record<HistoryState, HistoryFilterValues>> = {
  history: allToday,
  empty: { ...allToday, providers: [] },
  "no-match": { ...allToday, search: "deploy", place: "boat" },
  "date-menu": { ...allToday, menu: true },
  filtered: { ...allToday, ...historyFilter },
  current: allToday,
  "two-providers": { ...allToday, providers: ["boat", "second"] },
};

const lists: Readonly<Record<HistoryState, readonly Job[]>> = {
  history,
  empty: [],
  "no-match": [],
  "date-menu": history,
  filtered: filteredHistory,
  current: currentHistory,
  "two-providers": twoProviderHistory,
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

/** In the two-provider state, docker image opens its job on Second provider. */
const secondProviderJob = twoProviderHistory.find(
  (job) => job.where.kind === "provider" && job.where.provider === "second",
);
const twoProviderLinks: RowLinks = secondProviderJob
  ? { [secondProviderJob.id]: paths.jobSecondProvider }
  : {};

/** The rows that open a page other than the fixture page, in each state. */
function linksOf(state: HistoryState): RowLinks | undefined {
  if (state === "current") return currentLinks;
  return state === "two-providers" ? twoProviderLinks : undefined;
}

function footer(state: HistoryState, count: number): string {
  if (
    state === "history" ||
    state === "date-menu" ||
    state === "two-providers"
  ) {
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
            links={linksOf(state)}
          />
          <p className="more-row caption">{footer(state, jobs.length)}</p>
        </Panel>
      )}
    </AppShell>
  );
}
