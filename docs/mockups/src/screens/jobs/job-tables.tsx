// Job lists: a table on desktop and a dense list on mobile. Running and
// waiting jobs lead with a status dot. A running job shows where it runs, and
// a waiting job shows its label, because no machine holds it yet. Finished
// jobs show their result, where they ran, and how long they waited.

import type { Viewport } from "@mokly/mokly";
import { MockLink } from "@mokly/mokly";

import {
  DataTable,
  LoadingRows,
  type Column,
} from "../../components/data-table.tsx";
import { paths } from "../../components/paths.ts";
import {
  JobStateDot,
  JobStateLabel,
  WhereLabel,
} from "../../components/status.tsx";
import { dotJoin } from "../../fixtures/format.ts";
import { jobDetails } from "../../fixtures/job-details.ts";
import type { Job } from "../../fixtures/jobs.ts";
import { org } from "../../fixtures/org.ts";

export type JobListKind = "running" | "waiting" | "finished";

const columns: Record<JobListKind, readonly Column[]> = {
  running: [
    { key: "job", label: "Job", flex: 3 },
    { key: "where", label: "Where", flex: 2 },
    { key: "time", label: "Running for", width: 110, align: "right" },
  ],
  waiting: [
    { key: "job", label: "Job", flex: 3 },
    { key: "label", label: "Label", flex: 1 },
    { key: "time", label: "Waiting for", width: 110, align: "right" },
  ],
  finished: [
    { key: "state", label: "Result", width: 112 },
    { key: "job", label: "Job", flex: 3 },
    { key: "where", label: "Where", flex: 2 },
    { key: "waited", label: "Waited", width: 76, align: "right" },
    { key: "time", label: "Duration", width: 88, align: "right" },
    { key: "finished", label: "Finished", width: 80, align: "right" },
  ],
};

/** The job page of a finished job: the Boat job, the fallback, or the own. */
function detailPath(job: Job) {
  if (job.where.kind === "boat") return paths.jobBoat;
  const fallback = jobDetails.fallback;
  return job.name === fallback.name && job.ref === fallback.ref
    ? paths.jobFallback
    : paths.job;
}

function LabelName({ job }: { job: Job }) {
  return <span className="inline-code">{job.label}</span>;
}

function JobName({ job, kind }: { job: Job; kind: JobListKind }) {
  const context = dotJoin(job.workflow, `${org.login}/${job.repo} ${job.ref}`);
  return (
    <span className="job-name">
      {kind === "finished" ? (
        <MockLink to={detailPath(job)} className="job-name-title">
          {job.name}
        </MockLink>
      ) : (
        <span className="job-name-title">{job.name}</span>
      )}
      <span className="caption job-context">{context}</span>
    </span>
  );
}

function cell(job: Job, key: string, kind: JobListKind) {
  switch (key) {
    case "state":
      return <JobStateLabel state={job.state} />;
    case "job":
      return kind === "finished" ? (
        <JobName job={job} kind={kind} />
      ) : (
        <span className="job-cell">
          <JobStateDot state={job.state} />
          <JobName job={job} kind={kind} />
        </span>
      );
    case "where":
      return <WhereLabel where={job.where} slot={kind !== "finished"} />;
    case "label":
      return <LabelName job={job} />;
    case "waited":
      return <span className="duration">{job.waited}</span>;
    case "finished":
      return <span className="num muted">{job.finished}</span>;
    default:
      return <span className="duration">{job.time}</span>;
  }
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
  if (viewport === "mobile") return <JobRows kind={kind} jobs={jobs} />;
  return (
    <DataTable
      columns={columns[kind]}
      rows={jobs}
      rowKey={(job) => job.id}
      cell={(job, key) => cell(job, key, kind)}
    />
  );
}

/**
 * The dense mobile list. A live row leads with its state dot and shows where
 * the job runs; a finished row names its result, where it ran, and the wait.
 */
function JobRows({ kind, jobs }: { kind: JobListKind; jobs: readonly Job[] }) {
  const finished = kind === "finished";
  return (
    <ul className="job-rows">
      {jobs.map((job) => (
        <li key={job.id} className={`job-row${finished ? " is-finished" : ""}`}>
          {finished ? null : <JobStateDot state={job.state} />}
          <span className="job-row-main">
            <JobName job={job} kind={kind} />
            {finished ? (
              <span className="job-row-meta">
                <JobStateLabel state={job.state} />
                <WhereLabel where={job.where} />
                <span className="caption">waited {job.waited}</span>
              </span>
            ) : kind === "waiting" ? (
              <span className="job-row-meta">
                <LabelName job={job} />
              </span>
            ) : (
              <WhereLabel where={job.where} />
            )}
          </span>
          <span className="duration">{job.time}</span>
        </li>
      ))}
    </ul>
  );
}

/** A list that is still loading: a table on desktop, bar rows on mobile. */
export function LoadingJobList({ viewport }: { viewport: Viewport }) {
  if (viewport === "mobile") return <LoadingRows count={6} />;
  return (
    <DataTable
      columns={columns.running}
      rows={[]}
      rowKey={(_row, index) => String(index)}
      cell={() => null}
      loading
      loadingRowCount={6}
    />
  );
}
