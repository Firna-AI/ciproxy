// Job lists: a table on desktop and a list on mobile. Running and waiting jobs
// show where they run; finished jobs also show how long they waited. Inside
// a design direction, live rows lead with a status dot, and the mobile list
// is a dense list of its own.

import type { Viewport } from "@mokly/mokly";
import { MockLink } from "@mokly/mokly";
import { List, ListItem } from "@firna/ui/list";
import type { TableColumn } from "@firna/ui/table";

import { DataTable } from "../../components/data-table.tsx";
import { paths } from "../../components/paths.ts";
import {
  JobStateIcon,
  JobStateLabel,
  WhereLabel,
} from "../../components/status.tsx";
import type { Job } from "../../fixtures/jobs.ts";
import { org } from "../../fixtures/org.ts";
import { useDirection } from "../../theme/direction.ts";

export type JobListKind = "running" | "waiting" | "finished";

const columns: Record<JobListKind, TableColumn[]> = {
  running: [
    { key: "job", label: "Job", flex: 3 },
    { key: "where", label: "Where", flex: 2 },
    { key: "time", label: "Running for", width: 110, align: "right" },
  ],
  waiting: [
    { key: "job", label: "Job", flex: 3 },
    { key: "time", label: "Waiting for", width: 110, align: "right" },
  ],
  finished: [
    { key: "state", label: "Result", width: 120 },
    { key: "job", label: "Job", flex: 3 },
    { key: "where", label: "Where", flex: 2 },
    { key: "waited", label: "Waited", width: 80, align: "right" },
    { key: "time", label: "Duration", width: 90, align: "right" },
    { key: "finished", label: "Finished", width: 80, align: "right" },
  ],
};

function detailPath(job: Job) {
  return job.where.kind === "boat" ? paths.jobBoat : paths.job;
}

function JobName({ job, kind }: { job: Job; kind: JobListKind }) {
  const context = `${job.workflow} · ${org.login}/${job.repo} ${job.ref}`;
  return (
    <span className="job-name">
      {kind === "finished" ? (
        <MockLink to={detailPath(job)} className="job-name-title">
          {job.name}
        </MockLink>
      ) : (
        <span className="job-name-title">{job.name}</span>
      )}
      <span className="caption">{context}</span>
    </span>
  );
}

function cell(job: Job, key: string, kind: JobListKind, dot: boolean) {
  switch (key) {
    case "state":
      return <JobStateLabel state={job.state} />;
    case "job":
      return dot ? (
        <span className="job-cell">
          <JobStateIcon state={job.state} />
          <JobName job={job} kind={kind} />
        </span>
      ) : (
        <JobName job={job} kind={kind} />
      );
    case "where":
      return <WhereLabel where={job.where} slot={kind !== "finished"} />;
    case "waited":
      return <span className="num duration">{job.waited}</span>;
    case "finished":
      return <span className="num muted">{job.finished}</span>;
    default:
      return <span className="num duration">{job.time}</span>;
  }
}

function whereText(job: Job): string {
  if (job.where.kind === "own") return job.where.machine;
  if (job.where.kind === "boat") return "Boat";
  return "Waiting for a runner";
}

export function JobList({
  viewport,
  kind,
  jobs,
}: {
  viewport: Viewport;
  kind: JobListKind;
  jobs: readonly Job[];
}) {
  const direction = useDirection();
  if (viewport === "mobile" && direction) {
    return <JobRows kind={kind} jobs={jobs} />;
  }
  if (viewport === "mobile") {
    return (
      <List
        items={[...jobs]}
        itemKey={(job) => job.id}
        separators
        renderItem={(job) => (
          <ListItem
            leading={<JobStateIcon state={job.state} />}
            title={<JobName job={job} kind={kind} />}
            description={whereText(job)}
            trailing={<span className="num small">{job.time}</span>}
          />
        )}
      />
    );
  }
  const dot = direction !== null && kind !== "finished";
  return (
    <DataTable
      columns={columns[kind]}
      rows={[...jobs]}
      rowKey={(job) => job.id}
      cell={(job, key) => cell(job, key, kind, dot)}
    />
  );
}

/** The dense mobile list of a direction: state, job, where, and time. */
function JobRows({ kind, jobs }: { kind: JobListKind; jobs: readonly Job[] }) {
  return (
    <ul className="job-rows">
      {jobs.map((job) => (
        <li key={job.id} className="job-row">
          <JobStateIcon state={job.state} />
          <span className="job-row-main">
            <JobName job={job} kind={kind} />
            <WhereLabel where={job.where} />
          </span>
          <span className="duration">{job.time}</span>
        </li>
      ))}
    </ul>
  );
}

/** The table of a list that is still loading. */
export function LoadingJobList({ viewport }: { viewport: Viewport }) {
  return (
    <DataTable
      columns={
        viewport === "mobile" ? columns.running.slice(0, 2) : columns.running
      }
      rows={[]}
      rowKey={(_row, index) => String(index)}
      cell={() => null}
      loading
      loadingRowCount={6}
    />
  );
}
