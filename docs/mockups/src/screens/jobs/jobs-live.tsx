// The live queue: what waits for a runner and what runs now, on own machines
// and on Boat. GitHub gives a waiting job to any idle runner, so a waiting job
// has no machine yet; ciproxy shows how many runners it starts, and where.
// Every role sees the queue; only an Admin gets the actions of its notices.

import { Zap } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { ReadFailure, StateBlock } from "../../components/feedback.tsx";
import { Kpi, KpiRow } from "../../components/kpis.tsx";
import { LinkTabs, PageHeader, Panel } from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { canPay } from "../../components/role-actions.tsx";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { usageBanner } from "../../components/usage-banner.tsx";
import type { Role } from "../../fixtures/members.ts";
import { type LiveScene, liveScenes, type LiveState } from "./live-scenes.ts";
import { liveNotice } from "./live-notices.tsx";
import { JobList, LoadingJobList } from "./job-tables.tsx";
import "./jobs.css";

export const jobTabs = [
  { label: "Live", to: paths.jobs, live: true },
  { label: "History", to: paths.jobsHistory },
] as const;

export function JobsLiveScreen({
  viewport,
  state,
  role = "admin",
  accountMenu = false,
  workspaceMenu = false,
}: {
  viewport: Viewport;
  state: LiveState;
  /** The role of the signed-in person, which decides the actions. */
  role?: Role;
  /** The account menu of the avatar is open. */
  accountMenu?: boolean;
  /** The menu of the workspace switch is open. */
  workspaceMenu?: boolean;
}) {
  const scene = liveScenes[state];
  return (
    <AppShell
      viewport={viewport}
      tier={scene.tier}
      active="jobs"
      role={role}
      accountMenu={accountMenu}
      workspaceMenu={workspaceMenu}
      banner={
        state === "starts-passed"
          ? usageBanner("free-over", "other", canPay(role))
          : undefined
      }
    >
      <PageHeader title="Jobs" />
      <LinkTabs tabs={jobTabs} active={paths.jobs} />
      {state === "error" ? (
        <Panel>
          <ReadFailure subject="jobs" retry={paths.jobs} />
        </Panel>
      ) : (
        <>
          {state === "loading" ? null : <LiveFigures scene={scene} />}
          {liveNotice(state, scene, role)}
          <LiveLists viewport={viewport} state={state} scene={scene} />
        </>
      )}
    </AppShell>
  );
}

/** The figures of the queue; the waiting count only when ciproxy sees it. */
function LiveFigures({ scene }: { scene: LiveScene }) {
  return (
    <KpiRow>
      <Kpi label="Running" value={scene.running} />
      {scene.waitingUnknown ? null : (
        <Kpi label="Waiting" value={scene.waiting} />
      )}
      <Kpi
        label="Own slots in use"
        value={scene.ownBusy}
        valueFormat={(value) =>
          scene.slotsUnknown ? "Unknown" : `${value} of ${scene.ownSlots}`
        }
      />
      <Kpi label="On Boat" value={scene.onBoat} />
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
  if (state === "quiet-no-warm") {
    return (
      <Panel>
        <StateBlock icon={Zap} title="No job runs now">
          Jobs with <span className="inline-code">runs-on: ciproxy</span> show
          here when they wait or run.
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
