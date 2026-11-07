// Readers for Setup: its organizations, labels, and first job.

import {
  array,
  boolean,
  decimal,
  integer,
  object,
  oneOf,
  optional,
  parser,
  ParseError,
  string,
  time,
} from "./parse-core.ts";
import { readActivation } from "./parse-links.ts";
import type {
  InstallationState,
  LabelFit,
  LabelInput,
  LabelView,
  NothingFits,
  OrganizationView,
  RepositorySelection,
  SetupJob,
  SetupPhase,
  SetupPlace,
  SetupView,
} from "./schema.ts";

const installationState = oneOf<InstallationState>({
  active: true,
  suspended: true,
  removed: true,
});

const nothingFits = oneOf<NothingFits>({ wait: true, largest_boat: true });

const setupPhase = oneOf<SetupPhase>({
  available: true,
  assigned: true,
  started: true,
  completed: true,
});

const setupPlace = oneOf<SetupPlace>({ machine: true, boat: true });

function access(value: unknown): RepositorySelection {
  const fields = object(value, "repository access");
  if (fields.mode === "all_private") return { mode: "all_private" };
  if (fields.mode === "selected") {
    return { mode: "selected", ids: array(fields.ids, decimal) };
  }
  throw new ParseError("repository access");
}

export function readOrganization(value: unknown): OrganizationView {
  const fields = object(value, "organization");
  return {
    access: access(fields.access),
    activation: readActivation(fields.activation),
    delete_after: optional(fields.delete_after, time),
    github_id: string(fields.github_id),
    id: string(fields.id),
    installation_id: string(fields.installation_id),
    login: string(fields.login),
    manage_url: string(fields.manage_url),
    state: installationState(fields.state),
  };
}

function fit(value: unknown): LabelFit {
  const fields = object(value, "label fit");
  return {
    eligible_machines: integer(fields.eligible_machines),
    fitting_machines: integer(fields.fitting_machines),
    maximum_jobs: optional(fields.maximum_jobs, decimal),
    minimum_jobs: optional(fields.minimum_jobs, decimal),
    no_machine_fits: boolean(fields.no_machine_fits),
    unavailable_machines: integer(fields.unavailable_machines),
  };
}

function labelInput(value: unknown): LabelInput {
  const fields = object(value, "label");
  return {
    allow_boat: boolean(fields.allow_boat),
    allow_machines: boolean(fields.allow_machines),
    disk_gb: integer(fields.disk_gb),
    memory_gb: integer(fields.memory_gb),
    name: string(fields.name),
    vcpus: integer(fields.vcpus),
    when_nothing_fits: nothingFits(fields.when_nothing_fits),
  };
}

function label(value: unknown): LabelView {
  const fields = object(value, "label view");
  return {
    fit: fit(fields.fit),
    id: string(fields.id),
    label: labelInput(fields.label),
  };
}

function job(value: unknown): SetupJob {
  const fields = object(value, "job");
  return {
    assigned_at: optional(fields.assigned_at, time),
    completed_at: optional(fields.completed_at, time),
    id: string(fields.id),
    name: string(fields.name),
    phase: setupPhase(fields.phase),
    place: optional(fields.place, setupPlace),
    repository: string(fields.repository),
    result: optional(fields.result, string),
    started_at: optional(fields.started_at, time),
    workflow_ref: string(fields.workflow_ref),
  };
}

export const parseSetup = parser((value): SetupView => {
  const fields = object(value, "setup");
  return {
    current_image_disk_bytes: decimal(fields.current_image_disk_bytes),
    first_job: optional(fields.first_job, job),
    labels: array(fields.labels, label),
    machines: integer(fields.machines),
    organizations: array(fields.organizations, readOrganization),
    waiting_for_machine: boolean(fields.waiting_for_machine),
  };
});
