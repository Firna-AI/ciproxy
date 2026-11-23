// The fit line of the label dialog, which says, before the save, where jobs
// of the size run now: how many fit on one machine at a time, or that no
// machine fits the size, with the place of the order that runs it, why that
// place cannot take a job now, or that no machine and no size of a provider
// of the order fits it. The job counts, the provider fits, and the routing
// of the label come from the API fit; this module only puts them into words.
// The routing gives the same place and the same wait reason as the label
// list and the Jobs notices. A size that no machine fits is named by its
// full request, never by what a machine or a provider size offers. A
// provider that is not connected or not available has no known sizes, so
// the line never says that its sizes do not fit. When the order leaves own
// machines out, the line says so in place of their fit. A macOS label counts
// the Macs only. The sizes in words are in settings-label-sizes.ts.

import type { LabelFit, LabelInput, ProviderLabelFit } from "../api/schema.ts";
import { isMacLabel } from "./platform.ts";
import {
  capitalized,
  holdText,
  providerName,
  providerNames,
} from "./providers.ts";
import { hasMachines, placeName } from "./settings-label-places.ts";
import { requestPhrase } from "./settings-label-sizes.ts";

/** A caption, or a warning whose title can stand alone. */
export type FitLine =
  | { kind: "caption"; text: string }
  | { kind: "warning"; title: string; body: string | null };

/** "1 job", "3 jobs", or "2 to 3 jobs". */
export function jobsText(minimum: string, maximum: string): string {
  const unit = Number(maximum) === 1 ? "job" : "jobs";
  return minimum === maximum
    ? `${maximum} ${unit}`
    : `${minimum} to ${maximum} ${unit}`;
}

function plural(count: number, one: string, many: string): string {
  return `${count} ${count === 1 ? one : many}`;
}

/** The machines of a label: Linux machines, or Macs. */
type Machines = { one: string; many: string };

const linuxMachines: Machines = { one: "machine", many: "machines" };
const macs: Machines = { one: "Mac", many: "Macs" };

/** The jobs of the size that each fitting machine runs at a time. */
function machineFit(fit: LabelFit, noun: Machines = linuxMachines): string {
  const { eligible_machines: eligible, fitting_machines: fitting } = fit;
  const unknown = fit.unavailable_machines;
  const minimum = fit.minimum_jobs ?? null;
  const maximum = fit.maximum_jobs ?? null;
  if (minimum === null || maximum === null) {
    return `Your ${noun.many} run jobs of this size.`;
  }
  const jobs = jobsText(minimum, maximum);
  if (fitting === eligible) {
    return `Each of your ${noun.many} runs ${jobs} of this size at a time.`;
  }
  const unknownCapacity =
    unknown > 0
      ? ` The capacity of ${plural(unknown, noun.one, noun.many)} is not known yet.`
      : "";
  return `${jobs} of this size at a time on ${fitting} of your ${eligible} ${noun.many}.${unknownCapacity}`;
}

/**
 * The line of a macOS label, which runs on the Macs only. A Mac runs at most
 * 2 jobs at a time, and the API counts that limit in the fit.
 */
function macFit(label: LabelInput, fit: LabelFit): FitLine {
  if (fit.fitting_machines > 0) {
    return { kind: "caption", text: machineFit(fit, macs) };
  }
  if (fit.unavailable_machines > 0) {
    return {
      kind: "caption",
      text: "The capacity of your Macs is not known yet.",
    };
  }
  const wait = "Jobs with this label wait until a Mac that fits connects.";
  return {
    kind: "warning",
    title: `No Mac of yours fits a job of ${requestPhrase(label)}`,
    body:
      fit.eligible_machines === 0
        ? `No connected Mac can run jobs now. ${wait}`
        : wait,
  };
}

const noMachine = "No connected machine can run jobs now.";

/** The sentence, after the fact that no machine is connected when none is. */
function afterMachines(fit: LabelFit, sentence: string | null): string | null {
  if (fit.eligible_machines > 0) return sentence;
  return sentence === null ? noMachine : `${noMachine} ${sentence}`;
}

/** A provider of the order whose sizes ciproxy reads now. */
function sizesKnown(provider: ProviderLabelFit): boolean {
  const hold = provider.hold ?? null;
  if (hold === "not_connected" || hold === "unavailable") return false;
  return (
    provider.fit.state !== "unavailable" && provider.fit.state !== "disabled"
  );
}

/** A provider of the order whose sizes are not known yet. */
function sizesUnknown(provider: ProviderLabelFit): boolean {
  const hold = provider.hold ?? null;
  return (
    provider.fit.state === "unavailable" &&
    hold !== "not_connected" &&
    hold !== "unavailable" &&
    hold !== "off"
  );
}

/** The line of a Linux size that no own machine fits. */
function noMachineFits(label: LabelInput, fit: LabelFit): FitLine {
  const request = requestPhrase(label);
  const title = `No machine of yours fits a job of ${request}`;
  const warning = (heading: string, body: string | null): FitLine => ({
    kind: "warning",
    title: heading,
    body,
  });
  const { routing } = fit;
  if (routing.kind === "places") {
    const [first] = routing.places;
    const place = first ? placeName(first, "linux_x64") : "a provider";
    return warning(title, `Jobs with this label run on ${place}.`);
  }
  if (routing.kind === "waits" && routing.reason.kind === "provider") {
    const reason = holdText(routing.reason.provider, routing.reason.hold);
    return warning(
      title,
      `${capitalized(reason)}, so jobs with this label wait.`,
    );
  }
  const unknown = fit.providers.filter(sizesUnknown);
  if (unknown.length > 0) {
    const names = providerNames(
      unknown.map((item) => item.provider),
      "and",
    );
    return warning(title, `${names} sizes are not known yet.`);
  }
  const known = fit.providers.filter(sizesKnown);
  if (known.length === 0) {
    return warning(
      title,
      afterMachines(
        fit,
        "Jobs with this label wait until a machine that fits connects.",
      ),
    );
  }
  const names = providerNames(known.map((item) => item.provider));
  return warning(
    `No machine of yours and no ${names} size fits a job of ${request}`,
    routing.kind === "largest"
      ? `Jobs with this label run on the largest size of ${providerName(routing.provider)}.`
      : afterMachines(fit, null),
  );
}

/** The line of the dialog that says, before the save, where jobs run. */
export function fitLine(label: LabelInput, fit: LabelFit): FitLine {
  if (isMacLabel(label)) return macFit(label, fit);
  if (!hasMachines(label.order)) {
    return {
      kind: "caption",
      text: "Your machines are not in the order of this label.",
    };
  }
  if (fit.fitting_machines > 0) {
    return { kind: "caption", text: machineFit(fit) };
  }
  if (fit.unavailable_machines > 0) {
    return {
      kind: "caption",
      text: "The capacity of your machines is not known yet.",
    };
  }
  return noMachineFits(label, fit);
}
