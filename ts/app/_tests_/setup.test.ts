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
    preparing: false,
  });
  assert.equal(ready.steps[1]?.summary, "forge-01 · 6 slots");
  const preparing = setupModel(
    setup({ machines: 1, waiting_for_machine: false }),
    [machine({ state: "preparing" })],
    false,
    noBoat,
  );
  assert.equal(preparing.connected?.preparing, true);
  assert.deepEqual(
    [preparing.steps[1]?.status, preparing.steps[1]?.summary],
    ["done", "forge-01 · getting the job image"],
    "Setup goes on while the machine gets its first job image",
  );
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

test("a waiting first job names the machine whose job image it waits for, only when nothing else takes it", () => {
  const joined = { machines: 1, waiting_for_machine: false };
  const queued = job({
    phase: "available",
    assigned_at: null,
    started_at: null,
  });
  const available = setup({ ...joined, first_job: queued });
  const preparing = machine({ state: "preparing" });
  const second = {
    id: "30000000-0000-4000-8000-000000000002",
    hostname: "forge-02",
  };
  const skipped: BoatProgress = { connected: null, skipped: true };
  const boat: BoatProgress = {
    connected: "Connected to the Boat account northstar",
    skipped: false,
  };
  const wait = (
    view: typeof available,
    machines: Parameters<typeof setupModel>[1],
    progress: BoatProgress = skipped,
  ) => setupModel(view, machines, false, progress).imageWait;
  assert.equal(wait(available, [preparing]), "forge-01");
  assert.equal(wait(available, [preparing], noBoat), "forge-01");
  assert.equal(
    wait(available, [preparing], boat),
    null,
    "with Boat connected, routing starts the job on Boat",
  );
  assert.equal(
    wait(available, [preparing, machine(second)]),
    null,
    "another machine takes the job",
  );
  assert.equal(
    wait(available, [preparing, machine({ ...second, state: "preparing" })]),
    "forge-01",
  );
  assert.equal(wait(available, [machine()]), null);
  assert.equal(wait(setup(joined), [preparing]), null);
  for (const phase of ["assigned", "started"] as const) {
    assert.equal(
      wait(setup({ ...joined, first_job: job({ phase }) }), [preparing]),
      null,
    );
  }
});

test("Setup reads again while a machine gets its first job image, also after the first job ended", async () => {
  const ended = setup({
    machines: 1,
    waiting_for_machine: false,
    first_job: job({ phase: "completed", result: "success", place: "boat" }),
  });
  const preparing = machine({ state: "preparing" });
  const { app, schedule } = appWith(
    response(ended),
    response([preparing]),
    response(ended),
    response([preparing]),
    response(ended),
    response([machine()]),
  );
  const store = app.setup(workspaceId);
  await store.ensure();
  const stop = store.watch();
  schedule.fire();
  await settled();
  assert.equal(schedule.pending(), 1, "the machine still gets its image");
  schedule.fire();
  await settled();
  assert.equal(
    schedule.pending(),
    0,
    "nothing waits once the machine is ready",
  );
  stop();
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
