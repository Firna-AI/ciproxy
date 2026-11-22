// The slots of one machine, each with the job it runs or its warm runner. The
// name of a job opens its job page. A long job context wraps to a second line
// rather than being cut off. A free slot whose job the disk cannot hold says
// so. On a Mac, a free slot says so too while the Mac runs its 2 VMs.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import type { SlotState } from "../../fixtures/machines.ts";
import type { SlotJob } from "../../fixtures/slots.ts";

/** What a free slot says: free, or why it takes no job now. */
function freeText(index: number, usable: number, atLimit: boolean): string {
  if (atLimit) return "Free, the Mac runs its 2 VMs";
  return index < usable ? "Free" : "Free, no disk for a job";
}

export function MachineSlots({
  slots,
  jobs,
  usable = slots.length,
  vmLimit,
}: {
  slots: readonly SlotState[];
  /** The jobs of the busy slots, in their order. */
  jobs: readonly SlotJob[];
  /** The slots whose jobs of the default label the disk can hold. */
  usable?: number;
  /** The most VMs that the machine runs at a time: 2 on a Mac. */
  vmLimit?: number;
}) {
  const vms = slots.filter((slot) => slot !== "free").length;
  const atLimit = vmLimit !== undefined && vms >= vmLimit;
  let busyIndex = 0;
  return (
    <ul className="slot-list">
      {slots.map((slot, i) => {
        const job = slot === "busy" ? jobs[busyIndex++] : undefined;
        return (
          <li key={i} className="slot-list-row">
            <span className={`slot slot-${slot}`} aria-hidden="true" />
            <span className="caption slot-number">Slot {i + 1}</span>
            {job ? (
              <span className="slot-job">
                <TextLink to={paths.job}>{job.name}</TextLink>
                <span className="caption">{job.context}</span>
              </span>
            ) : (
              <span className="slot-job muted">
                {slot === "warm"
                  ? "Warm runner, ready"
                  : freeText(i, usable, atLimit)}
              </span>
            )}
            <span className="duration">{job?.time ?? ""}</span>
          </li>
        );
      })}
    </ul>
  );
}
