// The line of the label dialog that says, before the save, where jobs of the
// size run: how many fit on one machine at a time, that no machine fits the
// size, or that no machine and no Boat size fits it. When the machines fit
// different numbers, the line gives a range, as design call 41 says. It names
// the full request of the size with "fits", as the live queue does, and not
// the largest job that a machine or a Boat size takes, which the API does not
// give. Without a Boat account, the line keeps the fit of the machines and
// says that Boat is not connected; it checks no Boat size.

import { Notice } from "../../components/feedback.tsx";
import type { Machine } from "../../fixtures/machines.ts";
import {
  fitsOwnMachine,
  jobsPerMachine,
  type Label,
} from "../../fixtures/settings.ts";
import { fitsBoat } from "../../fixtures/sizes.ts";
import { requestPhrase } from "../jobs/place-fact-notices.tsx";

/** "1 job", "3 jobs", or "1 to 3 jobs". */
function jobsText(fewest: number, most: number): string {
  const unit = most === 1 ? "job" : "jobs";
  return fewest === most ? `${most} ${unit}` : `${fewest} to ${most} ${unit}`;
}

export function FitNote({
  label,
  machines,
  boat = true,
}: {
  label: Label;
  machines: readonly Machine[];
  /** The workspace has a Boat account. */
  boat?: boolean;
}) {
  if (fitsOwnMachine(label, machines)) {
    const { fewest, most } = jobsPerMachine(label, machines);
    return (
      <p className="caption">
        Each of your machines runs {jobsText(fewest, most)} of this size at a
        time.{boat ? null : " Boat is not connected."}
      </p>
    );
  }
  if (!boat) {
    return (
      <Notice
        tone="warning"
        title={`No machine of yours fits a job of ${requestPhrase(label)}`}
      >
        Boat is not connected, so jobs with this label wait until a machine that
        fits connects.
      </Notice>
    );
  }
  if (fitsBoat(label)) {
    return (
      <Notice
        tone="warning"
        title={`No machine of yours fits a job of ${requestPhrase(label)}`}
      >
        Jobs with this label run on Boat.
      </Notice>
    );
  }
  return (
    <Notice
      tone="warning"
      title={`No machine of yours and no Boat size fits a job of ${requestPhrase(label)}`}
    />
  );
}
