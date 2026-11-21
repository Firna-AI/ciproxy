// Readers for one job of the dashboard, as a busy slot of a machine names it,
// and for an actual VM size, which Boat sizes share. Each reader narrows an
// unknown value to its generated type, or throws a ParseError.

import {
  boolean,
  decimal,
  integer,
  object,
  oneOf,
  optional,
  ParseError,
  string,
  time,
} from "./parse-core.ts";
import type {
  JobCacheCounts,
  JobCacheSource,
  JobLabel,
  JobLocation,
  JobStatus,
  JobView,
  JobVmSize,
} from "./schema.ts";

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

/** An actual VM size: whole vCPUs, and bytes as decimal strings. */
export function readVmSize(value: unknown): JobVmSize {
  const fields = object(value, "VM size");
  return {
    disk_bytes: decimal(fields.disk_bytes),
    memory_bytes: decimal(fields.memory_bytes),
    vcpus: integer(fields.vcpus),
  };
}

function cacheCounts(value: unknown): JobCacheCounts {
  const fields = object(value, "cache counts");
  return {
    restored_bytes: decimal(fields.restored_bytes),
    restored_entries: decimal(fields.restored_entries),
  };
}

/** A whole number or null: a dimension that the request did not retain. */
function retained(value: unknown): number | null {
  return value === null ? null : integer(value);
}

function jobLabel(value: unknown): JobLabel {
  const fields = object(value, "job label");
  return {
    disk_gb: retained(fields.disk_gb),
    memory_gb: retained(fields.memory_gb),
    name: string(fields.name),
    vcpus: retained(fields.vcpus),
  };
}

function location(value: unknown): JobLocation {
  const fields = object(value, "job location");
  if (fields.kind === "machine") {
    return {
      first_slot: optional(fields.first_slot, integer),
      id: string(fields.id),
      kind: "machine",
      name: optional(fields.name, string),
    };
  }
  if (fields.kind === "boat") {
    return { kind: "boat", size: string(fields.size) };
  }
  throw new ParseError("job location");
}

/** Exact decimal seconds, such as "252" or "52.125", kept as the wire text. */
function seconds(value: unknown): string {
  const text = string(value);
  if (!/^\d+(\.\d+)?$/.test(text)) throw new ParseError("seconds");
  return text;
}

export function readJob(value: unknown): JobView {
  const fields = object(value, "job");
  return {
    actual_size: optional(fields.actual_size, readVmSize),
    cache_counts: optional(fields.cache_counts, cacheCounts),
    cache_source: optional(fields.cache_source, cacheSource),
    completed_at: optional(fields.completed_at, time),
    counts_as_start: boolean(fields.counts_as_start),
    git_ref: optional(fields.git_ref, string),
    github_url: optional(fields.github_url, string),
    head_branch: optional(fields.head_branch, string),
    id: string(fields.id),
    label: jobLabel(fields.label),
    location: optional(fields.location, location),
    name: string(fields.name),
    organization: string(fields.organization),
    organization_id: string(fields.organization_id),
    queued_at: optional(fields.queued_at, time),
    repository: string(fields.repository),
    result: optional(fields.result, string),
    routing_reason: optional(fields.routing_reason, string),
    runtime_seconds: optional(fields.runtime_seconds, seconds),
    started_at: optional(fields.started_at, time),
    status: jobStatus(fields.status),
    waited_seconds: optional(fields.waited_seconds, seconds),
    workflow: string(fields.workflow),
  };
}
