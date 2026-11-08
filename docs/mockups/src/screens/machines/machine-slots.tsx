// The slots of one machine, each with the job it runs or its warm runner. The
// name of a job opens its job page. A long job context wraps to a second line
// rather than being cut off. A free slot whose job the disk cannot hold says
// so.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import type { SlotState } from "../../fixtures/machines.ts";
import { ax42OneSlotJobs } from "../../fixtures/slots.ts";

export function MachineSlots({
  slots,
  usable = slots.length,
}: {
  slots: readonly SlotState[];
  /** The slots whose jobs of the label ciproxy the disk can hold. */
  usable?: number;
}) {
  let busyIndex = 0;
  return (
    <ul className="slot-list">
      {slots.map((slot, i) => {
        const job = slot === "busy" ? ax42OneSlotJobs[busyIndex++] : undefined;
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
                  : i < usable
                    ? "Free"
                    : "Free, no disk for a job"}
              </span>
            )}
            <span className="duration">{job?.time ?? ""}</span>
          </li>
        );
      })}
    </ul>
  );
}
