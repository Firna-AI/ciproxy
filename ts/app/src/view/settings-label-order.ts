// What each place of a label order does with a job of the label's size, as
// the order rows of the label dialog say it: how long a job waits for own
// machines, the provider size that runs it, or why jobs skip the place. A
// provider place says first when overflow is off, or when the provider is
// not connected or not available, and only then whether one of its sizes
// fits. The overflow wait shows on own machines only, and only when a place
// follows them. The facts come from the latest fit of the API. A move of a
// place changes no fact of a place, so the rows keep their words while the
// fit of the new order loads. A new size, or a place added or removed, waits
// for its own fit, and the rows say nothing until it answers.

import type {
  LabelFit,
  LabelInput,
  LabelPlace,
  ProviderLabelFit,
} from "../api/schema.ts";
import { formatGigabytes } from "./format.ts";
import { waitLabel } from "./overflow.ts";
import { isMacLabel, labelPlatform } from "./platform.ts";
import { capitalized, holdText, providerName } from "./providers.ts";
import { placeKey } from "./settings-label-places.ts";

/** A fit of the API with the label that it answers. */
export type AnsweredFit = { input: LabelInput; fit: LabelFit };

/** The parts of a label that the facts of each place depend on. */
function factsKey(input: LabelInput): string {
  const places = input.order.map(placeKey).sort().join(",");
  return [
    labelPlatform(input),
    input.vcpus,
    input.memory_gb,
    input.disk_gb,
    input.when_nothing_fits,
    places,
  ].join("/");
}

/** The fit whose facts hold for the draft, or null while none does. */
export function placeFacts(
  draft: LabelInput,
  shown: AnsweredFit | null,
): LabelFit | null {
  if (shown === null) return null;
  return factsKey(shown.input) === factsKey(draft) ? shown.fit : null;
}

/** "Jobs wait up to 30 seconds for a free slot", by the overflow wait. */
function waitCaption(seconds: number | null): string {
  if (seconds === null) {
    return "Jobs wait up to the overflow wait for a free slot";
  }
  if (seconds === 0) {
    return "Jobs go to the next place at once when no slot is free";
  }
  return `Jobs wait up to ${waitLabel(seconds)} for a free slot`;
}

function machineCaption(
  label: LabelInput,
  index: number,
  fit: LabelFit,
  waitSeconds: number | null,
): string {
  const mac = isMacLabel(label);
  const noun = mac ? "Mac" : "machine";
  if (fit.fitting_machines > 0) {
    if (mac) return "Jobs wait for a free Mac";
    return index < label.order.length - 1
      ? waitCaption(waitSeconds)
      : "Jobs wait for a free slot";
  }
  if (fit.unavailable_machines > 0) {
    return `The capacity of your ${mac ? "Macs" : "machines"} is not known yet`;
  }
  return fit.eligible_machines === 0
    ? `No ${noun} of yours can run jobs now`
    : `No ${noun} of yours fits this size`;
}

/** "16 vCPU and 32 GB": the size of a provider that runs the jobs. */
function sizePhrase(size: { vcpus: number; memory_bytes: string }): string {
  return `${size.vcpus} vCPU and ${formatGigabytes(size.memory_bytes)}`;
}

function providerCaption(facts: ProviderLabelFit): string | null {
  const name = providerName(facts.provider);
  const skip = (clause: string) => `${capitalized(clause)}, so jobs skip it`;
  const hold = facts.hold ?? null;
  if (hold === "off" || hold === "not_connected" || hold === "unavailable") {
    return skip(holdText(facts.provider, hold));
  }
  const { fit } = facts;
  if (fit.state === "disabled") return null;
  if (fit.state === "no_size_fits") return skip(`no ${name} size fits`);
  if (fit.state === "unavailable") return `${name} sizes are not known yet`;
  if (hold !== null) return skip(holdText(facts.provider, hold));
  if (fit.state === "largest_fallback") {
    return `Runs on its largest size, ${fit.size_name}, ${sizePhrase(fit.size)}`;
  }
  const size = facts.eligible_size ?? { name: fit.size_name, size: fit.size };
  return `Runs on ${size.name}, ${sizePhrase(size.size)}`;
}

/**
 * The words under a place of the order, or null while the fit that holds for
 * the draft has not answered.
 */
export function placeCaption(
  draft: LabelInput,
  place: LabelPlace,
  index: number,
  fit: LabelFit | null,
  waitSeconds: number | null,
): string | null {
  if (fit === null) return null;
  if (place.kind === "machine") {
    return machineCaption(draft, index, fit, waitSeconds);
  }
  const facts = fit.providers.find((item) => item.provider === place.provider);
  return facts === undefined ? null : providerCaption(facts);
}
