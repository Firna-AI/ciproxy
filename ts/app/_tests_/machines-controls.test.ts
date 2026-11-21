import assert from "node:assert/strict";
import test from "node:test";

import type { MachineChange } from "../src/state/machines-changes.ts";
import { warmControl } from "../src/view/machines-controls.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { slot } from "./machines-fixtures.ts";

test("the warm runner count shows the saved count in every machine state", () => {
  const runs: MachineChange[] = [];
  const changes = {
    run: (change: MachineChange) => {
      runs.push(change);
      return Promise.resolve(true);
    },
  };
  const idle = { status: "idle" } as const;
  const states = {
    healthy: machine({ occupancy: [slot(0, "warm"), slot(1, "free")] }),
    draining: machine({
      draining: true,
      occupancy: [slot(0, "job"), slot(1, "free")],
    }),
    offline: machine({ offline: true, health_old: true, occupancy: [] }),
    noKvm: machine({ kvm: false, slots: 0, occupancy: [] }),
    noSlots: machine({ slots: 0, occupancy: [] }),
  };
  for (const [state, view] of Object.entries(states)) {
    const control = warmControl(
      { ...view, warm_runners: 2 },
      idle,
      changes,
      true,
    );
    assert.equal(control.value, 2, `${state} shows the saved count`);
    control.onChange?.(3);
  }
  assert.deepEqual(
    runs,
    Array<MachineChange>(5).fill({ kind: "warm-runners", count: 3 }),
  );
  const saving = warmControl(
    machine({ offline: true, warm_runners: 2 }),
    { status: "busy", change: { kind: "warm-runners", count: 3 } },
    changes,
    true,
  );
  assert.deepEqual([saving.value, saving.saving], [3, true]);
  assert.equal(
    warmControl(machine({ warm_runners: 2 }), idle, changes, false).onChange,
    null,
  );
});
