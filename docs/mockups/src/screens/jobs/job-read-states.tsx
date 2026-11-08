// The job page before it has a job: while the read runs, when the read fails,
// and when the API has no such job in the history of the plan. The last case
// covers an old link and a job of another workspace. The breadcrumb leads
// back to History in each state.

import { SearchX } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { SkeletonBar, SkeletonGroup } from "@firna/ui/skeleton";

import { LinkAction } from "../../components/actions.tsx";
import { ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import "./jobs.css";
import "./job-detail.css";

export const jobReadStates = ["loading", "error", "not-found"] as const;
export type JobReadState = (typeof jobReadStates)[number];

const crumbs = [{ label: "Jobs", to: paths.jobsHistory }] as const;

function LoadingJob() {
  return (
    <div className="job-detail">
      <Panel title="Where it ran">
        <SkeletonGroup direction="column" gap={12}>
          <SkeletonBar width="60%" height={18} />
          <SkeletonBar width="80%" height={14} />
          <SkeletonBar width="50%" height={14} />
        </SkeletonGroup>
      </Panel>
      <Panel title="Details">
        <SkeletonGroup direction="column" gap={12}>
          <SkeletonBar width="90%" height={14} />
          <SkeletonBar width="90%" height={14} />
          <SkeletonBar width="90%" height={14} />
          <SkeletonBar width="90%" height={14} />
        </SkeletonGroup>
      </Panel>
    </div>
  );
}

export function JobReadScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: JobReadState;
}) {
  return (
    <AppShell viewport={viewport} tier="team" active="jobs">
      <PageHeader crumbs={crumbs} title="Job" />
      {state === "loading" ? <LoadingJob /> : null}
      {state === "error" ? (
        <Panel>
          <ReadFailure subject="the job" retry={paths.job} />
        </Panel>
      ) : null}
      {state === "not-found" ? (
        <Panel>
          <StateBlock
            icon={SearchX}
            title="This job is not in your history"
            action={
              <LinkAction to={paths.jobsHistory} tone="primary">
                Open History
              </LinkAction>
            }
          >
            It is older than the job history of your plan, or it belongs to
            another workspace.
          </StateBlock>
        </Panel>
      ) : null}
    </AppShell>
  );
}
