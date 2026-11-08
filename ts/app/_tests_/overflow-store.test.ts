import assert from "node:assert/strict";
import test from "node:test";

import { parseOverflowSettings } from "../src/api/parse-overflow.ts";
import type { OverflowPolicy, Problem, Success } from "../src/api/schema.ts";
import { session, workspaceId } from "../scripts/fixtures-core.ts";
import {
  boatWallets,
  notConnected,
  overflowSettings,
} from "../scripts/fixtures-boat.ts";
import { appWith, deferred, response } from "./support.ts";

const ok = { ok: true } satisfies Success;
const unavailable = {
  code: "unavailable",
  message: "Try again.",
} satisfies Problem;
const denied = {
  code: "access_denied",
  message: "This request is not permitted.",
} satisfies Problem;
const route = `/api/workspaces/${workspaceId}/overflow`;

function sent(init: RequestInit | undefined): unknown {
  return typeof init?.body === "string" ? JSON.parse(init.body) : undefined;
}

test("the settings parser keeps the generated fields and refuses a wrong shape", () => {
  const settings = overflowSettings();
  assert.deepEqual(parseOverflowSettings(settings), settings);
  const { account, spend, spend_limit_cents, ...required } = settings;
  assert.ok(account && spend && spend_limit_cents);
  assert.deepEqual(parseOverflowSettings(required), {
    ...required,
    account: null,
    spend: null,
    spend_limit_cents: null,
  });
  const { limit_reached, ...unflagged } = spend;
  assert.equal(limit_reached, false);
  assert.equal(
    parseOverflowSettings({ ...settings, spend: unflagged })?.spend
      ?.limit_reached,
    null,
  );
  for (const wrong of [
    { ...settings, spend: { ...spend, confirmed_cents: 4810 } },
    { ...settings, spend_limit_cents: "200.00" },
    { ...settings, account: { ...account, name: undefined } },
    { ...settings, next_month_at: "next month" },
    { ...settings, wait_seconds: "30" },
  ]) {
    assert.equal(parseOverflowSettings(wrong), undefined);
  }
});

test("a connection sends the key, the limit, and the chosen wallet with the CSRF token, and drops the key before it reads", async () => {
  const { app, calls } = appWith(
    response(session()),
    response(ok),
    response(overflowSettings()),
  );
  await app.session.ensure();
  const store = app.overflow(workspaceId);
  let callsAtAccept = -1;
  const result = await store.connect(
    { apiKey: "typed-key", limitCents: "20000", wallet: "boat-org-1002" },
    () => {
      callsAtAccept = calls.length;
    },
  );
  assert.deepEqual(result, { ok: true, value: ok });
  assert.equal(calls[1]?.url, `${route}/connection`);
  assert.equal(calls[1]?.init.method, "POST");
  assert.equal(
    new Headers(calls[1]?.init.headers).get("x-ciproxy-csrf"),
    "fixture-csrf-current",
  );
  assert.deepEqual(sent(calls[1]?.init), {
    api_key: "typed-key",
    spend_limit_cents: "20000",
    wallet: "boat-org-1002",
  });
  assert.equal(callsAtAccept, 2, "The key is dropped before the next read");
  assert.equal(calls[2]?.init.method, "GET");
  const state = store.state.get();
  assert.equal(
    state.status === "ready" ? state.settings.account?.name : null,
    "northstar",
  );
});

test("an accepted change whose read fails is never sent again: Try again only reads", async () => {
  const { app, calls } = appWith(
    response(ok),
    response(unavailable, 503),
    response(unavailable, 503),
    response(overflowSettings()),
  );
  const store = app.overflow(workspaceId);
  const result = await store.connect({
    apiKey: "k",
    limitCents: "100",
    wallet: "personal",
  });
  assert.equal(result.ok, true);
  const failed = store.state.get();
  assert.equal(failed.status, "failed");
  assert.equal(
    failed.status === "failed" ? failed.accepted : null,
    "connected",
  );
  await store.load();
  const still = store.state.get();
  assert.equal(still.status === "failed" ? still.accepted : null, "connected");
  await store.load();
  assert.equal(store.state.get().status, "ready");
  assert.deepEqual(
    calls.map(({ init }) => init.method),
    ["POST", "GET", "GET", "GET"],
  );
});

test("a refused connection saves nothing and reads nothing", async () => {
  const refused = {
    code: "boat_key_rejected",
    message: "Boat did not accept this key.",
  } satisfies Problem;
  const { app, calls } = appWith(
    response(notConnected()),
    response(refused, 422),
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  let accepted = false;
  const input = { apiKey: "k", limitCents: "100", wallet: "personal" };
  const result = await store.connect(input, () => {
    accepted = true;
  });
  assert.deepEqual(result, {
    ok: false,
    failure: { kind: "problem", status: 422, problem: refused },
  });
  assert.equal(accepted, false);
  assert.equal(calls.length, 2);
  const state = store.state.get();
  assert.equal(state.status === "ready" ? state.settings.account : "", null);
});

test("a policy save sends the complete policy, and a disconnect then reads again", async () => {
  const policy: OverflowPolicy = {
    enabled: false,
    wait_seconds: 60,
    spend_limit_cents: "25050",
  };
  const { app, calls } = appWith(
    response(ok),
    response(overflowSettings(policy)),
    response(ok),
    response(overflowSettings({ account: null })),
  );
  const store = app.overflow(workspaceId);
  await store.save(policy);
  assert.equal(calls[0]?.url, route);
  assert.equal(calls[0]?.init.method, "PUT");
  assert.deepEqual(sent(calls[0]?.init), policy);
  await store.disconnect();
  assert.equal(calls[2]?.url, `${route}/connection`);
  assert.equal(calls[2]?.init.method, "DELETE");
  assert.equal(calls[3]?.init.method, "GET");
  const state = store.state.get();
  assert.equal(state.status === "ready" ? state.settings.account : "", null);
});

test("a failed refresh keeps the last settings, and each workspace has its own store", async () => {
  const { app } = appWith(
    response(overflowSettings()),
    response(unavailable, 503),
  );
  const store = app.overflow(workspaceId);
  await store.ensure();
  await store.ensure();
  await store.load();
  const state = store.state.get();
  assert.equal(
    state.status === "ready" ? state.settings.account?.name : null,
    "northstar",
  );
  assert.equal(
    state.status === "ready" ? state.refreshFailure?.kind : null,
    "problem",
  );
  assert.notEqual(app.overflow("another-workspace"), store);
  assert.deepEqual(app.overflow("another-workspace").state.get(), {
    status: "loading",
  });
});

test("a key check sends only the key in the shared order, retries a stale CSRF token once, saves nothing, and reads nothing", async () => {
  const answer = deferred<Response>();
  const { app, calls } = appWith(
    response(session()),
    response(overflowSettings()),
    response(denied, 403),
    response(session({ csrf_token: "rotated-token" })),
    answer.promise,
  );
  await app.session.ensure();
  const store = app.overflow(workspaceId);
  await store.ensure();
  const checking = store.checkKey("typed-key");
  assert.equal(store.pending.get(), true, "The check holds the pending mark");
  answer.resolve(response(boatWallets()));
  assert.deepEqual(await checking, { ok: true, value: boatWallets() });
  assert.equal(calls[2]?.url, `${route}/wallets`);
  assert.equal(calls[2]?.init.method, "POST");
  assert.equal(
    new Headers(calls[2]?.init.headers).get("x-ciproxy-csrf"),
    "fixture-csrf-current",
  );
  assert.deepEqual(sent(calls[2]?.init), { api_key: "typed-key" });
  assert.deepEqual(
    calls.slice(3).map(({ url, init }) => `${init.method} ${url}`),
    ["GET /api/session", `POST ${route}/wallets`],
    "A stale token refreshes the session, and the check reads nothing after it",
  );
  assert.equal(
    new Headers(calls[4]?.init.headers).get("x-ciproxy-csrf"),
    "rotated-token",
  );
  assert.deepEqual(sent(calls[4]?.init), { api_key: "typed-key" });
  assert.equal(store.pending.get(), false);
  const state = store.state.get();
  assert.deepEqual(
    state.status === "ready" ? state.settings : null,
    overflowSettings(),
  );
});
