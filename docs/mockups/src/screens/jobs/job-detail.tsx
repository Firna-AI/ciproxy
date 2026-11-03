// One finished job: where it ran, why it ran there, and how long it waited.

import { Cloud, ExternalLink, Server } from "lucide-react";
import type { Viewport } from "@mokly/mokly";

import { Action } from "../../components/actions.tsx";
import {
  DetailList,
  DetailRow,
  PageHeader,
  Panel,
} from "../../components/page.tsx";
import { paths } from "../../components/paths.ts";
import { AppShell } from "../../components/shell/app-shell.tsx";
import { JobStateLabel } from "../../components/status.tsx";
import { org } from "../../fixtures/org.ts";
import "./jobs.css";

export const jobDetailStates = ["own", "boat"] as const;
export type JobDetailState = (typeof jobDetailStates)[number];

type Detail = {
  name: string;
  context: string;
  where: string;
  why: string;
  timeline: readonly [string, string][];
  waited: string;
  duration: string;
  cache: string;
  cacheSource: string;
};

const details: Record<JobDetailState, Detail> = {
  own: {
    name: "test (rust)",
    context: `${org.login}/mokly · CI · #1542 Add catalogue search`,
    where: "hetzner-ax42-1 · slot 3",
    why: "A warm runner was ready, so the job started at once.",
    timeline: [
      ["14:25:09", "Queued on GitHub"],
      ["14:25:09", "Taken by a warm runner on hetzner-ax42-1"],
      ["14:31:21", "Passed"],
    ],
    waited: "0 s",
    duration: "6m 12s",
    cache: "3 restored · 2.1 GB",
    cacheSource: "This machine",
  },
  boat: {
    name: "e2e (chromium) 3/4",
    context: `${org.login}/mokly · E2E · main`,
    where: "Boat",
    why: "Your machines were full, so the job started on Boat after 30 seconds.",
    timeline: [
      ["14:23:04", "Queued on GitHub"],
      ["14:23:04", "No free slot on your machines"],
      ["14:23:34", "Runner started on Boat"],
      ["14:23:35", "Taken by the Boat runner"],
      ["14:30:40", "Passed"],
    ],
    waited: "31 s",
    duration: "7m 05s",
    cache: "3 restored · 2.1 GB",
    cacheSource: "The GitHub cache",
  },
};

export function JobDetailScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: JobDetailState;
}) {
  const job = details[state];
  const Icon = state === "boat" ? Cloud : Server;
  return (
    <AppShell viewport={viewport} tier="team" active="jobs">
      <PageHeader
        crumbs={[{ label: "Jobs", to: paths.jobsHistory }]}
        title={job.name}
        status={<JobStateLabel state="passed" />}
        subtitle={job.context}
        actions={<Action icon={ExternalLink}>View on GitHub</Action>}
      />
      <div className="job-detail">
        <Panel title="Where it ran">
          <div className="where-ran">
            <span className="where-ran-icon">
              <Icon size={20} />
            </span>
            <div className="stack-tight">
              <p className="where-ran-title">{job.where}</p>
              <p className="small muted">{job.why}</p>
            </div>
          </div>
          <ol className="timeline">
            {job.timeline.map(([time, event]) => (
              <li key={`${time}-${event}`}>
                <span className="num caption">{time}</span>
                <span className="small">{event}</span>
              </li>
            ))}
          </ol>
        </Panel>
        <Panel title="Details">
          <DetailList>
            <DetailRow label="Label">
              <span className="inline-code">ciproxy</span>
            </DetailRow>
            <DetailRow label="Shape">2 vCPU · 8 GB</DetailRow>
            <DetailRow label="Waited">{job.waited}</DetailRow>
            <DetailRow label="Duration">{job.duration}</DetailRow>
            <DetailRow label="Cache">{job.cache}</DetailRow>
            <DetailRow label="Cache source">{job.cacheSource}</DetailRow>
            <DetailRow label="Starts">Counts as 1 start</DetailRow>
          </DetailList>
        </Panel>
      </div>
    </AppShell>
  );
}
