// The controls of the machine page that run a change: the warm runner count
// and the monthly cost. The count shows the saved count of the machine in
// every state, also while the machine drains, is offline, or has no slots,
// so a press changes the saved count and never a count of ready runners.
// While a change saves, the count shows the value that it saves.

import type { ApiFailure } from "../api/client.ts";
import type { MachineView } from "../api/schema.ts";
import type {
  ChangeSnapshot,
  MachineChangeStore,
} from "../state/machines-changes.ts";

export type WarmControl = {
  /** The count to show: the saved one, or the one that is saving. */
  value: number;
  saving: boolean;
  /** Null for a person who cannot change the count. */
  onChange: ((count: number) => void) | null;
};

export type CostEdit = {
  busy: boolean;
  /** The failure of the last save, while the form is open. */
  failure: ApiFailure | null;
  /** Saves the cents; resolves to whether the save succeeded. */
  save: (cents: string) => Promise<boolean>;
  /** Called when the form opens or closes, to clear an old failure. */
  reset: () => void;
};

/** Whether the change runs or ran the machine command `command`. */
export function isCommand(change: ChangeSnapshot, command: string): boolean {
  return (
    change.status !== "idle" &&
    change.change.kind === "command" &&
    change.change.command === command
  );
}

export function warmControl(
  machine: MachineView,
  change: ChangeSnapshot,
  changes: Pick<MachineChangeStore, "run">,
  admin: boolean,
): WarmControl {
  const pending =
    change.status === "busy" && change.change.kind === "warm-runners"
      ? change.change.count
      : null;
  return {
    value: pending ?? machine.warm_runners,
    saving: change.status === "busy",
    onChange: admin
      ? (count) => void changes.run({ kind: "warm-runners", count })
      : null,
  };
}

export function costEdit(
  change: ChangeSnapshot,
  changes: Pick<MachineChangeStore, "run" | "dismiss">,
): CostEdit {
  return {
    busy: change.status === "busy" && change.change.kind === "cost",
    failure:
      change.status === "failed" && change.change.kind === "cost"
        ? change.failure
        : null,
    save: (cents) => changes.run({ kind: "cost", cents }),
    reset: () => changes.dismiss(),
  };
}
