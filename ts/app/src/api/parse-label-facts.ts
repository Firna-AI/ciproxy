// Readers for the nested facts of a label fit: where the jobs of a label can
// run now, the offer of each eligible machine, and the Boat size that fits
// or that the account starts at most. The label readers in parse-setup.ts
// call them.

import {
  boolean,
  decimal,
  object,
  oneOf,
  optional,
  ParseError,
  string,
} from "./parse-core.ts";
import { readVmSize } from "./parse-jobs.ts";
import type {
  BoatLabelFit,
  LabelBoatSize,
  LabelMachineFit,
  LabelPlacement,
} from "./schema.ts";

export const readPlacement = oneOf<LabelPlacement>({
  machines_then_boat: true,
  machines_only: true,
  boat_only: true,
  waits: true,
  unavailable: true,
});

export function readBoatFit(value: unknown): BoatLabelFit {
  const fields = object(value, "Boat fit");
  switch (fields.state) {
    case "unavailable":
      return { state: "unavailable" };
    case "disabled":
      return { state: "disabled" };
    case "no_size_fits":
      return { state: "no_size_fits" };
    case "fitting":
      return {
        refused: boolean(fields.refused),
        size: readVmSize(fields.size),
        size_name: string(fields.size_name),
        state: "fitting",
      };
    case "largest_fallback":
      return {
        refused: boolean(fields.refused),
        size: readVmSize(fields.size),
        size_name: string(fields.size_name),
        state: "largest_fallback",
      };
  }
  throw new ParseError("Boat fit");
}

export function readBoatSize(value: unknown): LabelBoatSize {
  const fields = object(value, "Boat size");
  return { name: string(fields.name), size: readVmSize(fields.size) };
}

export function readMachineFit(value: unknown): LabelMachineFit {
  const fields = object(value, "machine fit");
  return {
    concurrent_jobs: optional(fields.concurrent_jobs, decimal),
    hostname: string(fields.hostname),
    id: string(fields.id),
    job_disk_bytes: optional(fields.job_disk_bytes, decimal),
    memory_bytes: decimal(fields.memory_bytes),
    required_root_disk_bytes: optional(
      fields.required_root_disk_bytes,
      decimal,
    ),
    slots: decimal(fields.slots),
    vcpus: decimal(fields.vcpus),
  };
}
