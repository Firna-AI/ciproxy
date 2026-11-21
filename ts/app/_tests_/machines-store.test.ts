import assert from "node:assert/strict";
import test from "node:test";

import type { Problem } from "../src/api/schema.ts";
import { createAddTracker, joinedMachines } from "../src/state/machines-add.ts";
import { machinesServices } from "../src/state/machines-services.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { appWith, deferred, response, settled } from "./support.ts";

const machinesUrl = `/api/workspaces/${workspaceId}/machines`;
const first = machine({ id: "m-1", hostname: "forge-01" });
const second = machine({ id: "m-2", hostname: "forge-02" });
const unavailable = {
  code: "unavailable",
  message: "The agent connection is busy.",
} satisfies Problem;

function ids(snapshot: { status: string; machines?: { id: string }[] }) {
  return snapshot.machines?.map((item) => item.id) ?? null;
}

test("a failed refresh keeps the last machines, and only the newest read counts", async () => {
  const old = deferred<Response>();
  const { app, calls } = appWith(
    response([first]),
    old.promise,
    response([first, second]),
    response(unavailable, 503),
  );
  const store = machinesServices(app).machines(workspaceId);
  await store.load();
  assert.deepEqual(ids(store.state.get()), ["m-1"]);
  const stale = store.load();
  await store.load();
  old.resolve(response([]));
  await stale;
  assert.deepEqual(ids(store.state.get()), ["m-1", "m-2"]);
  await store.load();
  const current = store.state.get();
  assert.deepEqual(
    current.status === "ready" ? [ids(current), current.refreshFailure] : null,
    [["m-1", "m-2"], { kind: "problem", status: 503, problem: unavailable }],
  );
  assert.equal(calls[0]?.url, machinesUrl);
});

test("a change runs once at a time and reads the machines again before it is done", async () => {
  const command = deferred<Response>();
  const { app, calls } = appWith(
    command.promise,
    response([{ ...first, draining: true }]),
  );
  const stores = machinesServices(app);
  const changes = stores.changes(workspaceId, "m-1");
  const running = changes.run({ kind: "command", command: "drain" });
  assert.equal(
    await changes.run({ kind: "command", command: "remove" }),
    false,
  );
  assert.equal(calls.length, 1);
  command.resolve(response({ connected: false }));
  assert.equal(await running, true);
  assert.deepEqual(
    calls.map((call) => [call.init.method, call.url]),
    [
      ["POST", `${machinesUrl}/m-1/actions/drain`],
      ["GET", machinesUrl],
    ],
  );
  assert.deepEqual(changes.state.get(), {
    status: "done",
    change: { kind: "command", command: "drain" },
    connected: false,
  });
  const listed = stores.machines(workspaceId).state.get();
  assert.equal(
    listed.status === "ready" ? listed.machines[0]?.draining : null,
    true,
  );
});

test("warm runners and the cost send their exact bodies, and a failure stays visible", async () => {
  const { app, calls } = appWith(
    response({ ok: true }),
    response([first]),
    response(unavailable, 503),
  );
  const changes = machinesServices(app).changes(workspaceId, "m-1");
  await changes.run({ kind: "warm-runners", count: 3 });
  await changes.run({ kind: "cost", cents: "5340" });
  assert.deepEqual(
    calls.map((call) => [call.init.method, call.url, call.init.body]),
    [
      [
        "PUT",
        `${machinesUrl}/m-1/warm-runners`,
        JSON.stringify({ warm_runners: 3 }),
      ],
      ["GET", machinesUrl, undefined],
      [
        "PUT",
        `${machinesUrl}/m-1/cost`,
        JSON.stringify({ monthly_cost_cents: "5340" }),
      ],
    ],
  );
  assert.deepEqual(changes.state.get(), {
    status: "failed",
    change: { kind: "cost", cents: "5340" },
    failure: { kind: "problem", status: 503, problem: unavailable },
  });
});

test("a removed machine leaves the list at once, and an older read cannot bring it back", async () => {
  const late = deferred<Response>();
  const { app, calls } = appWith(
    response([first, second]),
    late.promise,
    response({ connected: true }),
  );
  const stores = machinesServices(app);
  const store = stores.machines(workspaceId);
  await store.load();
  const pending = store.load();
  assert.equal(
    await stores
      .changes(workspaceId, "m-1")
      .run({ kind: "command", command: "remove" }),
    true,
  );
  late.resolve(response([first, second]));
  await pending;
  assert.deepEqual(ids(store.state.get()), ["m-2"]);
  assert.equal(calls.length, 3);
});

test("Add a machine takes its baseline from the first fresh read only", async () => {
  const fresh = deferred<Response>();
  const { app } = appWith(
    response([first]),
    response(unavailable, 503),
    fresh.promise,
    response([first, second]),
  );
  const stores = machinesServices(app);
  const store = stores.machines(workspaceId);
  await store.load();
  const tracker = createAddTracker(store);
  const stop = tracker.start();
  await settled();
  assert.equal(tracker.baseline.get(), null);
  assert.equal(tracker.failed.get(), true);
  const retry = tracker.retry();
  assert.equal(tracker.failed.get(), false);
  fresh.resolve(response([first]));
  await retry;
  assert.deepEqual([...(tracker.baseline.get() ?? [])], ["m-1"]);
  await store.load();
  assert.deepEqual(
    joinedMachines(store.state.get(), tracker.baseline.get()).map(
      (item) => item.hostname,
    ),
    ["forge-02"],
  );
  stop();
});
