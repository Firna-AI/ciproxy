// Add a machine waits for a machine that joins while the page is open. The
// page reads the machines when it opens, and the IDs of that read are the
// machines that were there before. The page shows the install command only
// after that read, so a machine that installs fast can never hide in it. A
// later read with an ID outside them is a machine that just connected. A read
// that failed, or a read from before the page opened, never sets the
// baseline, so an older machine never shows as new.

import type { MachineView } from "../api/schema.ts";
import type { MachinesSnapshot, MachinesStore } from "./machines.ts";
import { createStore, type Store } from "./store.ts";

export type AddTracker = {
  /** The machine IDs of the first fresh read, or null before it. */
  readonly baseline: Store<ReadonlySet<string> | null>;
  /** Whether the read for the baseline failed; Try again clears it. */
  readonly failed: Store<boolean>;
  /** Read the machines now and watch for the baseline. Returns the stop. */
  start(): () => void;
  /** Read again after a failed read for the baseline. */
  retry(): Promise<void>;
};

/** The add page polls faster than the list, as Setup does while it waits. */
export const addPollInterval = 5_000;

export function createAddTracker(machines: MachinesStore): AddTracker {
  const baseline = createStore<ReadonlySet<string> | null>(null);
  const failed = createStore(false);
  let before: MachinesSnapshot | null = null;

  const take = () => {
    if (baseline.get() !== null) return;
    const current = machines.state.get();
    if (current === before) return;
    if (current.status === "failed") {
      failed.set(true);
      return;
    }
    if (current.status !== "ready") return;
    if (current.refreshFailure !== null) {
      failed.set(true);
      return;
    }
    failed.set(false);
    baseline.set(new Set(current.machines.map((machine) => machine.id)));
  };

  return {
    baseline,
    failed,
    start() {
      before = machines.state.get();
      const unsubscribe = machines.state.subscribe(take);
      void machines.load();
      return unsubscribe;
    },
    async retry() {
      failed.set(false);
      await machines.load();
    },
  };
}

/** The machines that joined after the baseline, in the order of the read. */
export function joinedMachines(
  snapshot: MachinesSnapshot,
  baseline: ReadonlySet<string> | null,
): MachineView[] {
  if (baseline === null || snapshot.status !== "ready") return [];
  return snapshot.machines.filter((machine) => !baseline.has(machine.id));
}
