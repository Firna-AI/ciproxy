// Physical fit and routing facts in the generated label contract.

import {
  array,
  boolean,
  decimal,
  object,
  oneOf,
  optional,
  ParseError,
  string,
} from "./parse-core.ts";
import { readVmSize } from "./parse-jobs.ts";
import { readProvider } from "./parse-overflow.ts";
import type {
  LabelMachineFit,
  LabelPlace,
  LabelRouting,
  LabelWaitReason,
  ProviderCatalogSize,
  ProviderHold,
  ProviderLabelFit,
  ProviderSizeFit,
} from "./schema.ts";

const providerHold = oneOf<ProviderHold>({
  off: true,
  not_connected: true,
  unavailable: true,
  key_rejected: true,
  spend_limit: true,
  costs_unknown: true,
  retry_after: true,
  billing_required: true,
  size_refused: true,
});

export function readLabelPlace(value: unknown): LabelPlace {
  const fields = object(value, "label place");
  switch (fields.kind) {
    case "machine":
      return { kind: "machine" };
    case "overflow":
      return { kind: "overflow", provider: readProvider(fields.provider) };
    default:
      throw new ParseError("label place");
  }
}

function labelWait(value: unknown): LabelWaitReason {
  const fields = object(value, "label wait");
  switch (fields.kind) {
    case "empty_order":
    case "nothing_fits":
    case "unavailable":
      return { kind: fields.kind };
    case "provider":
      return {
        kind: "provider",
        provider: readProvider(fields.provider),
        hold: providerHold(fields.hold),
      };
    default:
      throw new ParseError("label wait");
  }
}

export function readLabelRouting(value: unknown): LabelRouting {
  const fields = object(value, "label routing");
  switch (fields.kind) {
    case "places":
      return { kind: "places", places: array(fields.places, readLabelPlace) };
    case "largest":
      return { kind: "largest", provider: readProvider(fields.provider) };
    case "waits":
      return { kind: "waits", reason: labelWait(fields.reason) };
    default:
      throw new ParseError("label routing");
  }
}

function sizeFit(value: unknown): ProviderSizeFit {
  const fields = object(value, "provider fit");
  switch (fields.state) {
    case "unavailable":
    case "disabled":
    case "no_size_fits":
      return { state: fields.state };
    case "fitting":
    case "largest_fallback":
      return {
        refused: boolean(fields.refused),
        size: readVmSize(fields.size),
        size_name: string(fields.size_name),
        state: fields.state,
      };
    default:
      throw new ParseError("provider fit");
  }
}

function catalogSize(value: unknown): ProviderCatalogSize {
  const fields = object(value, "provider size");
  return { name: string(fields.name), size: readVmSize(fields.size) };
}

export function readProviderFit(value: unknown): ProviderLabelFit {
  const fields = object(value, "label provider fit");
  return {
    provider: readProvider(fields.provider),
    connected: optional(fields.connected, boolean),
    fit: sizeFit(fields.fit),
    eligible_size: optional(fields.eligible_size, catalogSize),
    largest_size: optional(fields.largest_size, catalogSize),
    required_root_disk_bytes: optional(
      fields.required_root_disk_bytes,
      decimal,
    ),
    hold: optional(fields.hold, providerHold),
  };
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
