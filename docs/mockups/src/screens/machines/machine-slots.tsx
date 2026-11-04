// The slots of one machine, each with the job it runs or its warm runner. A
// long job context wraps to a second line rather than being cut off.

import type { SlotState } from "../../fixtures/machines.ts";
import { ax42OneSlotJobs } from "../../fixtures/slots.ts";

export function MachineSlots({ slots }: { slots: readonly SlotState[] }) {
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
                <span className="strong">{job.name}</span>
                <span className="caption">{job.context}</span>
              </span>
            ) : (
              <span className="slot-job muted">
                {slot === "warm" ? "Warm runner, ready" : "Free"}
              </span>
            )}
            <span className="duration">{job?.time ?? ""}</span>
          </li>
        );
      })}
    </ul>
  );
}
