// The line of the label dialog that says, before the save, where jobs of the
// size run: how many fit on one machine at a time, or that no machine fits
// the size, with the provider of the order that runs it, why that provider
// cannot take a job now, or that no machine and no size of a provider of the
// order fits it. When the machines fit different numbers, the line gives a
// range, as design call 41 says. It names the full request of the size with
// "fits", as the live queue does, and not the largest job that a machine or
// a provider size takes, which the API does not give. A provider that is not
// connected has no known sizes, so the line never names it. When the order
// leaves the own machines out, the line says so in place of their fit. A
// macOS label counts the Macs only.

import { Notice } from "../../components/feedback.tsx";
import { capitalized, holdText } from "../../components/hold-text.ts";
import { providersOf } from "../../fixtures/label-order.ts";
import { holdOf, type PlaceContext } from "../../fixtures/placement.ts";
import { providerName, providerNames } from "../../fixtures/providers.ts";
import {
  fitsOwnMachine,
  jobsPerMachine,
  type Label,
} from "../../fixtures/settings.ts";
import { fitsProvider } from "../../fixtures/sizes.ts";
import { requestPhrase } from "../jobs/place-fact-notices.tsx";

/** "1 job", "3 jobs", or "1 to 3 jobs". */
function jobsText(fewest: number, most: number): string {
  const unit = most === 1 ? "job" : "jobs";
  return fewest === most ? `${most} ${unit}` : `${fewest} to ${most} ${unit}`;
}

export function FitNote({
  label,
  context,
}: {
  label: Label;
  context: PlaceContext;
}) {
  const macos = label.platform === "macos";
  if (!label.order.includes("machines")) {
    return (
      <p className="caption">
        Your machines are not in the order of this label.
      </p>
    );
  }
  if (fitsOwnMachine(label, context.machines)) {
    const { fewest, most } = jobsPerMachine(label, context.machines);
    return (
      <p className="caption">
        Each of your {macos ? "Macs" : "machines"} runs {jobsText(fewest, most)}{" "}
        of this size at a time.
      </p>
    );
  }
  if (macos) {
    return (
      <Notice
        tone="warning"
        title={`No Mac of yours fits a job of ${requestPhrase(label)}`}
      >
        Jobs with this label wait until a Mac that fits connects.
      </Notice>
    );
  }
  const known = providersOf(label.order).filter(
    (provider) => holdOf(provider, context) !== "not-connected",
  );
  const fitting = known.filter((provider) => fitsProvider(provider, label));
  const noMachine = `No machine of yours fits a job of ${requestPhrase(label)}`;
  const runner = fitting.find((provider) => !holdOf(provider, context));
  if (runner) {
    return (
      <Notice tone="warning" title={noMachine}>
        Jobs with this label run on {providerName(runner)}.
      </Notice>
    );
  }
  const held = fitting[0];
  const hold = held ? holdOf(held, context) : undefined;
  if (held && hold) {
    return (
      <Notice tone="warning" title={noMachine}>
        {capitalized(holdText(held, hold))}, so jobs with this label wait.
      </Notice>
    );
  }
  if (known.length === 0) {
    return (
      <Notice tone="warning" title={noMachine}>
        Jobs with this label wait until a machine that fits connects.
      </Notice>
    );
  }
  return (
    <Notice
      tone="warning"
      title={`No machine of yours and no ${providerNames(known)} size fits a job of ${requestPhrase(label)}`}
    />
  );
}
