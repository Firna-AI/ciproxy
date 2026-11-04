// One finished job: where it ran, why it ran there, and how long it waited.

import type { ReactNode } from "react";
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
import { onBoat, own, type Where } from "../../fixtures/jobs.ts";
import { org } from "../../fixtures/org.ts";
import { labelNamed } from "../../fixtures/settings.ts";
import { sizeText } from "../../fixtures/sizes.ts";
import "./jobs.css";
import "./job-detail.css";

export const jobDetailStates = ["own", "boat"] as const;

/** Both jobs use the default label. */
const jobLabel = labelNamed("ciproxy");
export type JobDetailState = (typeof jobDetailStates)[number];

type Detail = {
  name: string;
  context: string;
  where: Where;
  why: string;
  timeline: readonly [string, ReactNode][];
  waited: string;
  duration: string;
  cache: string;
  cacheSource: string;
};

const details: Record<JobDetailState, Detail> = {
  own: {
    name: "test (rust)",
    context: `${org.login}/mokly · CI · #1542 Add catalogue search`,
    where: own(1, 3),
    why: "A warm runner was ready, so the job started at once.",
    timeline: [
      ["14:25:09", "Queued on GitHub"],
      [
        "14:25:09",
        <>
          Taken by a warm runner on{" "}
          <span className="mono nowrap">hetzner-ax42-1</span>
        </>,
      ],
      ["14:31:21", "Passed"],
    ],
    waited: "0s",
    duration: "6m 12s",
    cache: "3 restored · 2.1 GB",
    cacheSource: "This machine",
  },
  boat: {
    name: "e2e (chromium) 3/4",
    context: `${org.login}/mokly · E2E · main`,
    where: onBoat,
    why: "Your machines were full, so the job started on Boat after 30 seconds.",
    timeline: [
      ["14:23:04", "Queued on GitHub"],
      ["14:23:04", "No free slot on your machines"],
      ["14:23:34", "Runner started on Boat"],
      ["14:23:35", "Taken by the Boat runner"],
      ["14:30:40", "Passed"],
    ],
    waited: "31s",
    duration: "7m 05s",
    cache: "3 restored · 2.1 GB",
    cacheSource: "The GitHub cache",
  },
};

function WhereTitle({ where }: { where: Where }) {
  if (where.kind !== "own") return "Boat";
  return (
    <>
      <span className="mono">{where.machine}</span>
      <span className="nowrap"> · slot {where.slot}</span>
    </>
  );
}

export function JobDetailScreen({
  viewport,
  state,
}: {
  viewport: Viewport;
  state: JobDetailState;
}) {
  const job = details[state];
  const Icon = job.where.kind === "boat" ? Cloud : Server;
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
            <span className="icon-tile">
              <Icon size={16} />
            </span>
            <div className="stack-tight">
              <p className="where-ran-title">
                <WhereTitle where={job.where} />
              </p>
              <p className="small muted">{job.why}</p>
            </div>
          </div>
          <ol className="timeline">
            {job.timeline.map(([time, event], i) => (
              <li key={`${time}-${i}`}>
                <span className="mono timeline-time">{time}</span>
                <span className="small">{event}</span>
              </li>
            ))}
          </ol>
        </Panel>
        <Panel title="Details">
          <DetailList>
            <DetailRow label="Label">
              <span className="inline-code">{jobLabel.name}</span>
            </DetailRow>
            <DetailRow label="Size">{sizeText(jobLabel)}</DetailRow>
            <DetailRow label="Waited">
              <span className="duration">{job.waited}</span>
            </DetailRow>
            <DetailRow label="Duration">
              <span className="duration">{job.duration}</span>
            </DetailRow>
            <DetailRow label="Cache">{job.cache}</DetailRow>
            <DetailRow label="Cache source">{job.cacheSource}</DetailRow>
            <DetailRow label="Starts">Counts as 1 start</DetailRow>
          </DetailList>
        </Panel>
      </div>
    </AppShell>
  );
}
