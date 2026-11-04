// The line of the label dialog that says, before the save, where jobs of the
// size run: how many fit on one machine at a time, that no machine fits the
// size, or that no machine and no Boat size fits it. When the machines fit
// different numbers, the line gives a range, as design call 41 says.

import { Notice } from "../../components/feedback.tsx";
import { ax42, machineSlots, slotSize } from "../../fixtures/hardware.ts";
import type { Machine } from "../../fixtures/machines.ts";
import {
  fitsOwnMachine,
  jobsPerMachine,
  type Label,
} from "../../fixtures/settings.ts";
import { fitsBoat, largestBoatSize, sizePhrase } from "../../fixtures/sizes.ts";

/** The part of one Hetzner AX42 that jobs can use. */
const machineOffer = sizePhrase({
  vcpu: machineSlots(ax42) * slotSize.vcpu,
  memoryGb: machineSlots(ax42) * slotSize.memoryGb,
});

/** "1 job", "3 jobs", or "1 to 3 jobs". */
function jobsText(fewest: number, most: number): string {
  const unit = most === 1 ? "job" : "jobs";
  return fewest === most ? `${most} ${unit}` : `${fewest} to ${most} ${unit}`;
}

export function FitNote({
  label,
  machines,
}: {
  label: Label;
  machines: readonly Machine[];
}) {
  if (fitsOwnMachine(label, machines)) {
    const { fewest, most } = jobsPerMachine(label, machines);
    return (
      <p className="caption">
        Each of your machines runs {jobsText(fewest, most)} of this size at a
        time.
      </p>
    );
  }
  if (fitsBoat(label)) {
    return (
      <Notice tone="warning" title={`No machine fits ${sizePhrase(label)}`}>
        Jobs with this label run on Boat. Your machines offer {machineOffer}{" "}
        each.
      </Notice>
    );
  }
  return (
    <Notice
      tone="warning"
      title={`No machine and no Boat size fits ${sizePhrase(label)}`}
    >
      Your machines offer {machineOffer} each, and the largest Boat size offers{" "}
      {sizePhrase(largestBoatSize)}.
    </Notice>
  );
}
