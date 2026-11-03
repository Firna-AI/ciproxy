// The live queue: what waits for a runner and what runs now, on own machines
// and on Boat. GitHub gives a waiting job to any idle runner, so a waiting job
// has no machine yet; ciproxy shows how many runners it starts, and where.

import type { ReactNode } from "react";
import { Zap } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { Notice, ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { Kpi, KpiRow } from "../../components/kpis.tsx";
import { LinkTabs, PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { liveTrends } from "../../fixtures/trends.ts";
import { type LiveScene, liveScenes, type LiveState } from "./live-scenes.ts";
import { JobList, LoadingJobList } from "./job-tables.tsx";
import "./jobs.css";
import "./jobs-dir.css";

export const jobTabs = [
  { label: "Live", to: paths.jobs, live: true },
  { label: "History", to: paths.jobsHistory },
] as const;

function notices(state: LiveState, scene: LiveScene): ReactNode {
  if (state === "peak") {
    return (
      <Notice
        tone="info"
        title={`Your machines are full. ciproxy is starting ${scene.waiting} runners on Boat.`}
      >
        A job starts on Boat when it has waited 30 seconds for a free slot.
      </Notice>
    );
  }
  if (state === "spend-limit") {
    return (
      <Notice
        tone="warning"
        title="Boat reached its $200 limit for November"
        actions={
          <LinkAction to={paths.overflowLimitReached} size="sm">
            Raise the limit
          </LinkAction>
        }
      >
        Jobs wait for a free slot on your machines until 1 December.
      </Notice>
    );
  }
  return null;
}

function startsPassedBanner(): ReactNode {
  return (
    <Notice
      tone="warning"
      title="mokly-ai used the 2,000 starts of Free"
      actions={
        <>
          <LinkAction to={paths.usageFreeOver} size="sm">
            Plan and usage
          </LinkAction>
          <LinkAction to={paths.plans} tone="primary" size="sm">
            Choose Team
          </LinkAction>
        </>
      }
    >
      Jobs keep running. Team includes 25,000 starts and 3 own nodes for $49 a
      month.
    </Notice>
  );
}

export function JobsLiveScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: LiveState;
}) {
  const scene = liveScenes[state];
  return (
    <AppShell
      viewport={viewport}
      tier={scene.tier}
      active="jobs"
      banner={state === "starts-passed" ? startsPassedBanner() : undefined}
    >
      <PageHeader title="Jobs" />
      <LinkTabs tabs={jobTabs} active={paths.jobs} />
      {state === "error" ? (
        <Panel>
          <ReadFailure retry={paths.jobs} />
        </Panel>
      ) : (
        <>
          {state === "loading" ? null : (
            <LiveFigures scene={scene} steady={state === "steady"} />
          )}
          {notices(state, scene)}
          <LiveLists viewport={viewport} state={state} scene={scene} />
        </>
      )}
    </AppShell>
  );
}

function LiveFigures({ scene, steady }: { scene: LiveScene; steady: boolean }) {
  const trends = steady ? liveTrends.steady : undefined;
  return (
    <KpiRow>
      <Kpi label="Running" value={scene.running} trend={trends?.running} />
      <Kpi label="Waiting" value={scene.waiting} trend={trends?.waiting} />
      <Kpi
        label="Own slots in use"
        value={scene.ownBusy}
        valueFormat={(value) => `${value} of ${scene.ownSlots}`}
        trend={trends?.busySlots}
      />
      <Kpi label="On Boat" value={scene.onBoat} trend={trends?.onBoat} />
    </KpiRow>
  );
}

function LiveLists({
  viewport,
  state,
  scene,
}: {
  viewport: Viewport;
  state: LiveState;
  scene: LiveScene;
}) {
  if (state === "loading") {
    return (
      <Panel title="Running" flush>
        <LoadingJobList viewport={viewport} />
      </Panel>
    );
  }
  if (state === "quiet") {
    return (
      <Panel>
        <StateBlock icon={Zap} title={`${scene.warm} warm runners are ready`}>
          The next job with{" "}
          <span className="inline-code">runs-on: ciproxy</span> starts at once
          on one of your machines.
        </StateBlock>
      </Panel>
    );
  }
  return (
    <>
      {scene.waitingJobs.length > 0 ? (
        <Panel title={`Waiting · ${scene.waiting}`} flush>
          <JobList
            viewport={viewport}
            kind="waiting"
            jobs={scene.waitingJobs}
          />
          <MoreRow shown={scene.waitingJobs.length} total={scene.waiting} />
        </Panel>
      ) : null}
      <Panel title={`Running · ${scene.running}`} flush>
        <JobList viewport={viewport} kind="running" jobs={scene.runningJobs} />
        <MoreRow shown={scene.runningJobs.length} total={scene.running} />
      </Panel>
    </>
  );
}

function MoreRow({ shown, total }: { shown: number; total: number }) {
  if (total <= shown) return null;
  return <p className="more-row caption">{total - shown} more</p>;
}
