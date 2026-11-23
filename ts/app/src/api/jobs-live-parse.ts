// Readers for the live queue: the running and waiting jobs, the slot and
// runner figures, the fit of each waiting label, the provider facts, and the
// organizations whose jobs ciproxy cannot see. A figure that the API leaves
// out stays null, apart from zero.

import { readBannerField } from "./banner-reader.ts";
import { readJob } from "./jobs-parse.ts";
import type {
  JobOverflow,
  JobVisibility,
  JobsLivePage,
  ProviderRunningCount,
  StartingRunner,
  WaitingLabelFit,
} from "./jobs-types.ts";
import {
  array,
  boolean,
  decimal,
  integer,
  object,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";
import { readProvider } from "./parse-overflow.ts";
import { readLabel } from "./parse-setup.ts";

function waitingLabel(value: unknown): WaitingLabelFit {
  const fields = object(value, "waiting label");
  return {
    label_id: string(fields.label_id),
    label_name: string(fields.label_name),
    waiting_jobs: decimal(fields.waiting_jobs),
    can_edit: boolean(fields.can_edit),
    ask_admin: boolean(fields.ask_admin),
    current: optional(fields.current, readLabel),
    nothing_fits: optional(fields.nothing_fits, boolean),
  };
}

function overflow(value: unknown): JobOverflow {
  const fields = object(value, "overflow");
  return {
    provider: readProvider(fields.provider),
    connected: boolean(fields.connected),
    key_rejected: boolean(fields.key_rejected),
    billing_required: boolean(fields.billing_required),
    measured_complete: boolean(fields.measured_complete),
    next_month: time(fields.next_month),
    refused_sizes: array(fields.refused_sizes, string),
    retry_at: optional(fields.retry_at, time),
    limit_cents: optional(fields.limit_cents, decimal),
    counted_cents: optional(fields.counted_cents, decimal),
    limit_reached: optional(fields.limit_reached, boolean),
  };
}

function visibility(value: unknown): JobVisibility {
  const fields = object(value, "organization visibility");
  return {
    organization: string(fields.organization),
    suspended: boolean(fields.suspended),
    removed: boolean(fields.removed),
    github_url: string(fields.github_url),
  };
}

function startingRunner(value: unknown): StartingRunner {
  const fields = object(value, "starting runner");
  return {
    id: string(fields.id),
    label_id: string(fields.label_id),
    provider: optional(fields.provider, readProvider),
    machine_id: optional(fields.machine_id, string),
  };
}

function runningCount(value: unknown): ProviderRunningCount {
  const fields = object(value, "provider running count");
  return {
    provider: readProvider(fields.provider),
    running: decimal(fields.running),
  };
}

export const parseJobsLive = parser((value): JobsLivePage => {
  const fields = object(value, "live queue");
  return {
    running: array(fields.running, readJob),
    waiting: array(fields.waiting, readJob),
    waiting_count: optional(fields.waiting_count, decimal),
    waiting_labels: array(fields.waiting_labels, waitingLabel),
    running_by_provider: array(fields.running_by_provider, runningCount),
    warm_runners: decimal(fields.warm_runners),
    own_slots_total: optional(fields.own_slots_total, decimal),
    own_slots_used: optional(fields.own_slots_used, decimal),
    starting_runners: array(fields.starting_runners, startingRunner),
    visibility: array(fields.visibility, visibility),
    overflow: array(fields.overflow, overflow),
    overflow_enabled: boolean(fields.overflow_enabled),
    overflow_wait_seconds: integer(fields.overflow_wait_seconds),
    can_manage: boolean(fields.can_manage),
    banner: readBannerField(fields.banner),
  };
});
