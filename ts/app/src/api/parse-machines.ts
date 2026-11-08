// Readers for the machines of a workspace and the one-hour install command.

import {
  array,
  boolean,
  decimal,
  integer,
  object,
  oneOf,
  optional,
  parser,
  string,
  time,
} from "./parse-core.ts";
import type {
  CacheView,
  DiskView,
  InstallCommand,
  MachineView,
  SlotState,
  SlotView,
} from "./schema.ts";

const slotState = oneOf<SlotState>({ free: true, warm: true, job: true });

function cache(value: unknown): CacheView {
  const fields = object(value, "machine cache");
  return {
    capacity_bytes: decimal(fields.capacity_bytes),
    docker_disk_bytes: decimal(fields.docker_disk_bytes),
    hits_last_7_days: decimal(fields.hits_last_7_days),
    restores_last_7_days: decimal(fields.restores_last_7_days),
    used_bytes: decimal(fields.used_bytes),
  };
}

function disk(value: unknown): DiskView {
  const fields = object(value, "disk");
  return {
    capacity_bytes: decimal(fields.capacity_bytes),
    free_bytes: decimal(fields.free_bytes),
    path: string(fields.path),
  };
}

function slot(value: unknown): SlotView {
  const fields = object(value, "slot");
  return {
    index: decimal(fields.index),
    job_id: optional(fields.job_id, string),
    runner_id: optional(fields.runner_id, string),
    state: slotState(fields.state),
  };
}

function machine(value: unknown): MachineView {
  const fields = object(value, "machine");
  return {
    agent_version: string(fields.agent_version),
    cache: optional(fields.cache, cache),
    capacity_known: boolean(fields.capacity_known),
    concurrent_default_jobs: optional(fields.concurrent_default_jobs, decimal),
    disks: array(fields.disks, disk),
    draining: boolean(fields.draining),
    health_old: boolean(fields.health_old),
    hostname: string(fields.hostname),
    id: string(fields.id),
    kvm: boolean(fields.kvm),
    last_contact_at: optional(fields.last_contact_at, time),
    memory_bytes: decimal(fields.memory_bytes),
    monthly_cost_cents: optional(fields.monthly_cost_cents, decimal),
    occupancy: array(fields.occupancy, slot),
    offline: boolean(fields.offline),
    processor: string(fields.processor),
    selected_image_disk_bytes: optional(
      fields.selected_image_disk_bytes,
      decimal,
    ),
    slots: integer(fields.slots),
    threads: integer(fields.threads),
    warm_runners: integer(fields.warm_runners),
  };
}

export const parseMachines = parser((value) => array(value, machine));

export const parseInstallCommand = parser((value): InstallCommand => {
  const fields = object(value, "install command");
  return {
    command: string(fields.command),
    expires_at: time(fields.expires_at),
    id: string(fields.id),
    requirements: string(fields.requirements),
  };
});
