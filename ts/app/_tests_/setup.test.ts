import assert from "node:assert/strict";
import test from "node:test";

import type { Problem, SetupPhase } from "../src/api/schema.ts";
import { setupWaits } from "../src/state/setup.ts";
import {
  firstJob,
  setupModel,
  type BoatProgress,
} from "../src/view/setup-steps.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import {
  job,
  machine,
  organization,
  setup,
} from "../scripts/fixtures-setup.ts";
import { appWith, response, settled } from "./support.ts";

const noBoat: BoatProgress = { connected: null, skipped: false };

test("Setup uses actual machine capacity and KVM for its steps", () => {
  const initial = setupModel(setup(), [], false, noBoat);
  assert.deepEqual(
    initial.steps.map((step) => [step.key, step.status]),
    [
      ["github", "done"],
      ["machine", "active"],
      ["boat", "todo"],
      ["job", "todo"],
    ],
  );
  assert.deepEqual(
    [initial.done, initial.total, initial.complete],
    [1, 4, false],
  );
  const unavailable = setupModel(
    setup({ machines: 1 }),
    [machine({ kvm: false, hostname: "no-kvm" })],
    false,
    noBoat,
  );
  assert.equal(unavailable.noKvm, "no-kvm");
  assert.equal(unavailable.connected, null);
  const ready = setupModel(
    setup({ machines: 1, waiting_for_machine: false }),
    [machine()],
    false,
    noBoat,
  );
  assert.deepEqual(ready.connected, {
    name: "forge-01",
    facts: "12 threads · 64 GB · KVM ready · 6 slots",
    diskNote: "Its disk holds 2 jobs of ciproxy at a time.",
  });
  assert.deepEqual(
    ready.steps.map((step) => step.status),
    ["done", "done", "active", "todo"],
  );
  assert.equal(
    setupModel(setup({ machines: 1 }), [machine()], true, noBoat).steps[1]
      ?.status,
    "active",
  );
  assert.deepEqual(
    setupModel(
      setup({ organizations: [organization({ state: "suspended" })] }),
      [],
      false,
      noBoat,
    ).suspended,
    ["northstar-labs"],
  );
});

test("first job follows GitHub phases and only its success result reports a pass", () => {
  assert.deepEqual(firstJob(null), { kind: "waiting" });
  const phases: SetupPhase[] = ["available", "assigned", "started"];
  const texts = [
    "Build and test in northstar-labs/control is waiting for a runner…",
    "Build and test in northstar-labs/control is starting…",
    "Build and test in northstar-labs/control is running on your machine…",
  ];
  for (const [index, phase] of phases.entries()) {
    assert.deepEqual(firstJob(job({ phase })), {
      kind: "moving",
      text: texts[index],
    });
  }
  const passed = job({
    phase: "completed",
    result: "success",
    completed_at: "2026-11-06T12:07:22Z",
  });
  assert.deepEqual(firstJob(passed), {
    kind: "finished",
    passed: true,
    title: "Your first job passed",
    text: "Build and test in northstar-labs/control passed in 6m 12s on your machine.",
  });
  for (const result of ["failure", "cancelled", "timed_out", "skipped", null]) {
    const finished = firstJob(job({ phase: "completed", result }));
    assert.equal(finished.kind === "finished" ? finished.passed : null, false);
  }
  const complete = setup({
    machines: 1,
    waiting_for_machine: false,
    first_job: passed,
  });
  assert.equal(setupWaits(complete), false);
  assert.equal(setupModel(complete, [machine()], false, noBoat).complete, true);
});

test("Setup refresh errors retain the last view and recover without creating install tokens", async () => {
  const { app, calls } = appWith(
    response(setup()),
    response([]),
    response(
      { code: "unavailable", message: "Try again." } satisfies Problem,
      503,
    ),
    response([]),
    response(setup({ machines: 1, waiting_for_machine: false })),
    response([machine()]),
  );
  const store = app.setup(workspaceId);
  await store.ensure();
  await store.ensure();
  assert.equal(calls.length, 2);
  await store.load();
  const failed = store.state.get();
  assert.equal(failed.status === "ready" ? failed.setup.machines : null, 0);
  assert.equal(
    failed.status === "ready" ? failed.refreshFailure?.kind : null,
    "problem",
  );
  await store.load();
  const recovered = store.state.get();
  assert.equal(
    recovered.status === "ready" ? recovered.machines[0]?.hostname : null,
    "forge-01",
  );
  assert.equal(
    recovered.status === "ready" ? recovered.refreshFailure : undefined,
    null,
  );
  assert.deepEqual(
    calls.map(({ init }) => init.method),
    ["GET", "GET", "GET", "GET", "GET", "GET"],
  );
});

test("Setup polling stops when the first job completes and can be disposed", async () => {
  const { app, schedule } = appWith(
    response(setup()),
    response([]),
    response(
      setup({
        machines: 1,
        waiting_for_machine: false,
        first_job: job({ phase: "completed", result: "failure" }),
      }),
    ),
    response([machine()]),
  );
  const store = app.setup(workspaceId);
  await store.ensure();
  const stop = store.watch();
  assert.equal(schedule.pending(), 1);
  schedule.fire();
  await settled();
  assert.equal(schedule.pending(), 0);
  stop();
  schedule.hide(false);
  assert.equal(schedule.pending(), 0);
});
