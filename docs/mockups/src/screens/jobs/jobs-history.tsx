// Finished jobs with their result, where each ran, and how long each waited.

import { ChevronDown, ListChecks, Search } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { InputFrame } from "@firna/ui/input";
import { SegmentedControl } from "@firna/ui/segmented";

import { Action, noop } from "../../components/actions.tsx";
import { StateBlock } from "../../components/feedback.tsx";
import { LinkTabs, PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { history } from "../../fixtures/job-history.ts";
import { jobTabs } from "./jobs-live.tsx";
import { JobList } from "./job-tables.tsx";
import "./jobs.css";

export const historyStates = ["history", "empty"] as const;
export type HistoryState = (typeof historyStates)[number];

const whereOptions = [
  { label: "All", value: "all" },
  { label: "Own machines", value: "own" },
  { label: "Boat", value: "boat" },
] as const;

export function JobsHistoryScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: HistoryState;
}) {
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
      {state === "empty" ? (
        <Panel>
          <StateBlock icon={ListChecks} title="Finished jobs show here">
            Each job that used{" "}
            <span className="inline-code">runs-on: ciproxy</span>, with where it
            ran and how long it waited.
          </StateBlock>
        </Panel>
      ) : (
        <>
          <div className="history-filters">
            <span className="history-search">
              <InputFrame
                prefixIcon={Search}
                placeholder="Search jobs"
                value=""
                onChangeText={noop}
                accessibilityLabel="Search jobs"
              />
            </span>
            <SegmentedControl
              options={whereOptions}
              value="all"
              onChange={noop}
              accessibilityLabel="Where the job ran"
            />
            <Action icon={ChevronDown}>Today</Action>
          </div>
          <Panel flush>
            <JobList viewport={viewport} kind="finished" jobs={history} />
            <p className="more-row caption">
              10 of 612 jobs today · Team keeps 30 days of history
            </p>
          </Panel>
        </>
      )}
    </AppShell>
  );
}
