import assert from "node:assert/strict";
import test from "node:test";

import type { Success } from "../src/api/schema.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { overflowSettings } from "../scripts/fixtures-boat.ts";
import { appWith, deferred, response, settled } from "./support.ts";

const ok = { ok: true } satisfies Success;

function sent(init: RequestInit | undefined): unknown {
  return typeof init?.body === "string" ? JSON.parse(init.body) : undefined;
}

function methods(calls: { init: RequestInit }[]): (string | undefined)[] {
  return calls.map(({ init }) => init.method);
}

test("a wait save after an accepted key change waits for its read, then keeps the accepted limit", async () => {
  const read = deferred<Response>();
  const { app, calls } = appWith(
    response(overflowSettings()),
    response(ok),
    read.promise,
    response(ok),
    response(
      overflowSettings({ spend_limit_cents: "10000", wait_seconds: 60 }),
    ),
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  const connecting = store.connect({
    api_key: "replacement-key",
    spend_limit_cents: "10000",
    wallet: "personal",
  });
  await settled();
  assert.deepEqual(
    methods(calls),
    ["GET", "POST", "GET"],
    "The API accepted the key, and the read after it is held",
  );
  const saving = store.save({ wait_seconds: 60 });
  await settled();
  assert.deepEqual(
    methods(calls),
    ["GET", "POST", "GET"],
    "No PUT before the held read returns",
  );
  read.resolve(response(overflowSettings({ spend_limit_cents: "10000" })));
  await connecting;
  await saving;
  assert.deepEqual(methods(calls), ["GET", "POST", "GET", "PUT", "GET"]);
  assert.deepEqual(sent(calls[3]?.init), {
    enabled: true,
    wait_seconds: 60,
    spend_limit_cents: "10000",
  });
});

test("the pending mark covers a change through the read after it", async () => {
  const read = deferred<Response>();
  const { app } = appWith(
    response(overflowSettings()),
    response(ok),
    read.promise,
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  assert.equal(store.pending.get(), false);
  const disconnecting = store.disconnect();
  assert.equal(store.pending.get(), true);
  await settled();
  assert.equal(store.pending.get(), true, "The read keeps the mark");
  read.resolve(response(overflowSettings({ account: null })));
  await disconnecting;
  assert.equal(store.pending.get(), false);
});

test("a page visit during a change reads only after the change and its read", async () => {
  const post = deferred<Response>();
  const { app, calls } = appWith(
    response(overflowSettings()),
    post.promise,
    response(overflowSettings({ spend_limit_cents: "10000" })),
    response(overflowSettings({ spend_limit_cents: "10000" })),
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  const connecting = store.connect({
    api_key: "key",
    spend_limit_cents: "10000",
    wallet: "personal",
  });
  const visiting = store.load();
  await settled();
  assert.deepEqual(methods(calls), ["GET", "POST"], "The visit waits");
  post.resolve(response(ok));
  await connecting;
  await visiting;
  assert.deepEqual(methods(calls), ["GET", "POST", "GET", "GET"]);
  const state = store.state.get();
  assert.equal(
    state.status === "ready" ? state.settings.spend_limit_cents : null,
    "10000",
  );
});

test("a save builds the complete policy on the latest confirmed settings", async () => {
  const { app, calls } = appWith(
    response(overflowSettings({ enabled: false, wait_seconds: 15 })),
    response(ok),
    response(overflowSettings()),
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  await store.save({ spend_limit_cents: "25050" });
  assert.deepEqual(sent(calls[1]?.init), {
    enabled: false,
    wait_seconds: 15,
    spend_limit_cents: "25050",
  });
});

test("a save with no confirmed settings to build on sends nothing", async () => {
  const { app, calls } = appWith();
  const store = app.overflow(workspaceId);
  const result = await store.save({ wait_seconds: 60 });
  assert.equal(result, null);
  assert.equal(calls.length, 0);
  assert.equal(store.pending.get(), false);
});
