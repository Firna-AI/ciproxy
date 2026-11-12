// Readers for one job, the history, and the job page. A request dimension,
// a time, a place, or a cache fact that the API leaves out stays null, so a
// screen can tell an unknown value from zero. Seconds stay exact decimals.

import { readBannerField } from "./banner-reader.ts";
import type {
  JobCacheCounts,
  JobCacheSource,
  JobDetailPage,
  JobLabel,
  JobLocation,
  JobStatus,
  JobView,
  JobVmSize,
  JobsHistoryPage,
} from "./jobs-types.ts";
import {
  array,
  boolean,
  decimal,
  fraction,
  integer,
  object,
  oneOf,
  optional,
  parser,
  ParseError,
  string,
  time,
} from "./parse-core.ts";

const jobStatus = oneOf<JobStatus>({
  queued: true,
  running: true,
  completed: true,
});

const cacheSource = oneOf<JobCacheSource>({
  machine: true,
  github: true,
  machine_and_github: true,
});

function label(value: unknown): JobLabel {
  const fields = object(value, "job label");
  return {
    name: string(fields.name),
    vcpus: optional(fields.vcpus, integer),
    memory_gb: optional(fields.memory_gb, integer),
    disk_gb: optional(fields.disk_gb, integer),
  };
}

export function readVmSize(value: unknown): JobVmSize {
  const fields = object(value, "VM size");
  return {
    vcpus: integer(fields.vcpus),
    memory_bytes: decimal(fields.memory_bytes),
    disk_bytes: decimal(fields.disk_bytes),
  };
}

function location(value: unknown): JobLocation {
  const fields = object(value, "job place");
  if (fields.kind === "machine") {
    return {
      kind: "machine",
      id: string(fields.id),
      name: optional(fields.name, string),
      first_slot: optional(fields.first_slot, integer),
    };
  }
  if (fields.kind === "boat")
    return { kind: "boat", size: string(fields.size) };
  throw new ParseError("job place");
}

function cacheCounts(value: unknown): JobCacheCounts {
  const fields = object(value, "cache counts");
  return {
    restored_bytes: decimal(fields.restored_bytes),
    restored_entries: decimal(fields.restored_entries),
  };
}

export function readJob(value: unknown): JobView {
  const fields = object(value, "job");
  return {
    id: string(fields.id),
    organization_id: string(fields.organization_id),
    organization: string(fields.organization),
    name: string(fields.name),
    workflow: string(fields.workflow),
    repository: string(fields.repository),
    git_ref: optional(fields.git_ref, string),
    head_branch: optional(fields.head_branch, string),
    status: jobStatus(fields.status),
    result: optional(fields.result, string),
    queued_at: optional(fields.queued_at, time),
    started_at: optional(fields.started_at, time),
    completed_at: optional(fields.completed_at, time),
    counts_as_start: boolean(fields.counts_as_start),
    waited_seconds: optional(fields.waited_seconds, fraction),
    runtime_seconds: optional(fields.runtime_seconds, fraction),
    label: label(fields.label),
    location: optional(fields.location, location),
    actual_size: optional(fields.actual_size, readVmSize),
    routing_reason: optional(fields.routing_reason, string),
    github_url: optional(fields.github_url, string),
    cache_source: optional(fields.cache_source, cacheSource),
    cache_counts: optional(fields.cache_counts, cacheCounts),
  };
}

export const parseJobsHistory = parser((value): JobsHistoryPage => {
  const fields = object(value, "job history");
  return {
    jobs: array(fields.jobs, readJob),
    history_days: integer(fields.history_days),
    banner: readBannerField(fields.banner),
  };
});

export const parseJobDetail = parser((value): JobDetailPage => {
  const fields = object(value, "job page");
  return { job: readJob(fields.job), banner: readBannerField(fields.banner) };
});
