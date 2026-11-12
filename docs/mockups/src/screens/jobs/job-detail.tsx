// One finished job: where it ran, why it ran there, how long it waited, its
// label, and the VM size that ran it. A job on Boat names the Boat size,
// which Boat bills, with its whole disk as Boat states it, while a label
// states the free disk of its jobs. The cache counts show only when the
// machine cache handled the restores; a job on Boat, and a job that fell back
// to the GitHub cache, show only the cache source. When ciproxy has no saved
// request for a job, the label says so in place of a size, and the VM size
// comes from the VM that ran the job, never from the label as it is now.

import { CircleDashed, Cloud, ExternalLink, Server } from "lucide-react";
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
import { dotJoin, unbroken } from "../../fixtures/format.ts";
import {
  jobDetails,
  type CacheUse,
  type JobDetail,
  type JobDetailState,
} from "../../fixtures/job-details.ts";
import type { Where } from "../../fixtures/jobs.ts";
import { boatSizeText, sizeText, vmSizeText } from "../../fixtures/sizes.ts";
import "./jobs.css";
import "./job-detail.css";

/** The text in place of a routing reason that the API does not give. */
const noReason = "Why the job ran here is not available.";

function WhereTitle({ where }: { where: Where }) {
  if (where.kind === "none") return "No runner";
  if (where.kind !== "own") return "Boat";
  return (
    <>
      <span className="mono">{where.machine}</span>
      <span className="nowrap"> · slot {where.slot}</span>
    </>
  );
}

/**
 * The VM size that ran the job: the request on a machine, Boat's on Boat. For
 * an unknown request, the VM that ran it, with its whole VM disk, and a Boat
 * size without the claim that it fits the label.
 */
function VmSize({ job }: { job: JobDetail }) {
  if (!job.boatSize) {
    if (job.request !== "unknown") return <>{sizeText(job.request)}</>;
    return <>{job.vm ? vmSizeText(job.vm) : "Unknown"}</>;
  }
  return (
    <span className="detail-stack">
      <span>
        {dotJoin(`Boat ${job.boatSize.name}`, boatSizeText(job.boatSize))}
      </span>
      <span className="caption">
        {job.request === "unknown" || job.currentContract
          ? "Boat bills this size."
          : "The cheapest Boat size that fits the label and the image. Boat bills this size."}
      </span>
    </span>
  );
}

/**
 * The label of the job. On Boat, the size of the label shows under its name,
 * because the Boat size can be larger. An unknown request shows there too.
 */
function LabelValue({ job }: { job: JobDetail }) {
  const name = <span className="inline-code">{job.label}</span>;
  if (job.request !== "unknown" && !job.boatSize) return name;
  return (
    <span className="detail-stack">
      {name}
      <span className="caption">
        {job.request === "unknown"
          ? "Requested size unknown"
          : sizeText(job.request)}
      </span>
    </span>
  );
}

/** The caches that gave the entries that the machine cache restored. */
const machineSources = {
  machine: "This machine",
  both: "This machine and the GitHub cache",
} as const;

/**
 * The cache counts and the cache source. A job that restored entries from
 * both caches names both, and a job that restored no entry names no source,
 * because each text is then true for every job.
 */
function CacheRows({ cache, job }: { cache: CacheUse; job: JobDetail }) {
  if (cache.source === "unknown") {
    return <DetailRow label="Cache">Unknown</DetailRow>;
  }
  if (cache.source === "none") {
    return <DetailRow label="Cache">No entries restored</DetailRow>;
  }
  if (cache.source === "machine" || cache.source === "both") {
    return (
      <>
        <DetailRow label="Cache">
          {dotJoin(
            unbroken(`${cache.restored} restored`),
            unbroken(`${cache.sizeGb} GB`),
          )}
        </DetailRow>
        <DetailRow label="Cache source">
          {machineSources[cache.source]}
        </DetailRow>
      </>
    );
  }
  return (
    <DetailRow label="Cache source">
      {cache.fallback && job.where.kind === "own" ? (
        <span className="detail-stack">
          <span>The GitHub cache</span>
          <span className="caption">
            The cache of <span className="mono">{job.where.machine}</span> was
            not available, so the job used the GitHub cache.
          </span>
        </span>
      ) : (
        "The GitHub cache"
      )}
    </DetailRow>
  );
}

/** The rows of a job that a ciproxy runner took. */
function RanDetails({ job }: { job: JobDetail }) {
  return (
    <>
      <DetailRow label="VM size">
        <VmSize job={job} />
      </DetailRow>
      <DetailRow label="Waited">
        <span className="duration">{job.waited}</span>
      </DetailRow>
      <DetailRow label="Duration">
        <span className="duration">{job.duration}</span>
      </DetailRow>
      <CacheRows cache={job.cache} job={job} />
      <DetailRow label="Starts">Counts as 1 start</DetailRow>
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
  const job = jobDetails[state];
  const Icon =
    job.where.kind === "boat"
      ? Cloud
      : job.where.kind === "none"
        ? CircleDashed
        : Server;
  return (
    <AppShell viewport={viewport} tier="team" active="jobs">
      <PageHeader
        crumbs={[{ label: "Jobs", to: paths.jobsHistory }]}
        title={job.name}
        status={<JobStateLabel state={job.result ?? "passed"} />}
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
              <p className="small muted">{job.why ?? noReason}</p>
            </div>
          </div>
          <ol className="timeline">
            {job.timeline.map((event, i) => (
              <li key={`${event.time}-${i}`}>
                <span className="mono timeline-time">{event.time}</span>
                <span className="small">
                  {event.text}
                  {event.machine ? (
                    <>
                      {" "}
                      <span className="mono nowrap">{event.machine}</span>
                    </>
                  ) : null}
                </span>
              </li>
            ))}
          </ol>
        </Panel>
        <Panel title="Details">
          <DetailList>
            <DetailRow label="Label">
              <LabelValue job={job} />
            </DetailRow>
            {job.noRunner ? null : <RanDetails job={job} />}
          </DetailList>
        </Panel>
      </div>
    </AppShell>
  );
}
