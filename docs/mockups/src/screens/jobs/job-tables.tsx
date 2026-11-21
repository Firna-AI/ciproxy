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
import { paths, type CataloguePath } from "../../components/paths.ts";
import {
  JobStateDot,
  JobStateLabel,
  WhereLabel,
} from "../../components/status.tsx";
import { dotJoin } from "../../fixtures/format.ts";
import {
  jobDetails,
  jobDetailStates,
  type JobDetailState,
} from "../../fixtures/job-details.ts";
import { repositoryName, type Job } from "../../fixtures/jobs.ts";

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

/** The job page of each state of the job fixture. */
const detailPaths: Readonly<Record<JobDetailState, CataloguePath>> = {
  own: paths.job,
  boat: paths.jobBoat,
  fallback: paths.jobFallback,
  "mixed-cache": paths.jobMixedCache,
  "no-restores": paths.jobNoRestores,
  "unknown-request": paths.jobUnknownRequest,
  "unknown-request-boat": paths.jobUnknownRequestBoat,
  "no-runner": paths.jobNoRunner,
  "unknown-facts": paths.jobUnknownFacts,
  "current-own": paths.jobCurrentOwn,
  "current-boat": paths.jobCurrentBoat,
};

/**
 * The job page of a finished job: the page of the job with its name and ref,
 * the Boat job for another job on Boat, and the own job for the rest.
 */
function detailPath(job: Job): CataloguePath {
  const state = jobDetailStates.find(
    (key) =>
      jobDetails[key].name === job.name && jobDetails[key].ref === job.ref,
  );
  if (state) return detailPaths[state];
  return job.where.kind === "boat" ? paths.jobBoat : paths.job;
}

function LabelName({ job }: { job: Job }) {
  return <span className="inline-code">{job.label}</span>;
}

/** A page for some rows in place of the page that the fixtures give. */
export type RowLinks = Readonly<Record<string, CataloguePath>>;

function JobName({
  job,
  kind,
  links,
}: {
  job: Job;
  kind: JobListKind;
  links?: RowLinks;
}) {
  const context = dotJoin(
    job.workflow,
    `${repositoryName(job.repo)} ${job.ref}`,
  );
  return (
    <span className="job-name">
      {kind === "finished" ? (
        <MockLink
          to={links?.[job.id] ?? detailPath(job)}
          className="job-name-title"
        >
          {job.name}
        </MockLink>
      ) : (
        <span className="job-name-title">{job.name}</span>
      )}
      <span className="caption job-context">{context}</span>
    </span>
  );
}

/** Where a job runs or ran. A finished job without a place had no runner. */
function Where({ job, kind }: { job: Job; kind: JobListKind }) {
  if (kind === "finished" && job.where.kind === "none") {
    return <span className="muted">No runner</span>;
  }
  return <WhereLabel where={job.where} slot={kind !== "finished"} />;
}

function cell(job: Job, key: string, kind: JobListKind, links?: RowLinks) {
  switch (key) {
    case "state":
      return <JobStateLabel state={job.state} />;
    case "job":
      return kind === "finished" ? (
        <JobName job={job} kind={kind} links={links} />
      ) : (
        <span className="job-cell">
          <JobStateDot state={job.state} />
          <JobName job={job} kind={kind} />
        </span>
      );
    case "where":
      return <Where job={job} kind={kind} />;
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
  links,
}: {
  viewport: Viewport;
  kind: JobListKind;
  jobs: readonly Job[];
  /** Finished rows that open a page other than the fixture page. */
  links?: RowLinks;
}) {
  if (viewport === "mobile") {
    return <JobRows kind={kind} jobs={jobs} links={links} />;
  }
  return (
    <DataTable
      columns={columns[kind]}
      rows={jobs}
      rowKey={(job) => job.id}
      cell={(job, key) => cell(job, key, kind, links)}
    />
  );
}

/**
 * The dense mobile list. A live row leads with its state dot and shows where
 * the job runs; a finished row names its result, where it ran, and the wait.
 */
function JobRows({
  kind,
  jobs,
  links,
}: {
  kind: JobListKind;
  jobs: readonly Job[];
  links?: RowLinks;
}) {
  const finished = kind === "finished";
  return (
    <ul className="job-rows">
      {jobs.map((job) => (
        <li key={job.id} className={`job-row${finished ? " is-finished" : ""}`}>
          {finished ? null : <JobStateDot state={job.state} />}
          <span className="job-row-main">
            <JobName job={job} kind={kind} links={links} />
            {finished ? (
              <span className="job-row-meta">
                <JobStateLabel state={job.state} />
                <Where job={job} kind={kind} />
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
