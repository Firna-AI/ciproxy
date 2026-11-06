import assert from "node:assert/strict";
import test from "node:test";

import type { Problem, Success } from "../src/api/schema.ts";
import { sessionFreshFor } from "../src/state/session.ts";
import { storageKeys } from "../src/state/storage.ts";
import { fixtureTime, session } from "../scripts/fixtures-core.ts";
import { appWith, deferred, response } from "./support.ts";

const denied = {
  code: "access_denied",
  message: "Access changed.",
} satisfies Problem;
const ended = {
  code: "sign_in_required",
  message: "Sign in again.",
} satisfies Problem;
const success = { ok: true } satisfies Success;

test("sign out clears local access and a stale refresh cannot restore it", async () => {
  const stale = deferred<Response>();
  const { app } = appWith(
    response(session()),
    stale.promise,
    response(success),
  );
  await app.session.ensure();
  const refresh = app.session.refresh();
  await app.session.signOut();
  assert.deepEqual(app.session.state.get(), {
    status: "signed-out",
    reason: "none",
  });
  assert.equal(app.session.csrf(), null);
  assert.equal(app.env.local.get(storageKeys.signInEnds), null);
  stale.resolve(response(session()));
  await refresh;
  assert.deepEqual(app.session.state.get(), {
    status: "signed-out",
    reason: "none",
  });
});

test("401 ends access and an expired sign-in survives a stale request", async () => {
  const stale = deferred<Response>();
  const { app, setNow } = appWith(response(session()), stale.promise);
  await app.session.ensure();
  const refresh = app.session.refresh();
  setNow(Date.parse("2026-12-06T12:00:00Z"));
  app.session.report({
    ok: false,
    failure: { kind: "problem", status: 401, problem: ended },
  });
  stale.resolve(response(session()));
  await refresh;
  assert.deepEqual(app.session.state.get(), {
    status: "signed-out",
    reason: "ended",
  });
  assert.equal(app.session.csrf(), null);
});

test("403 mutations retry once only after a fresh session rotates CSRF", async () => {
  for (const rotated of [true, false]) {
    const current = session();
    const fresh = session({
      csrf_token: rotated ? "rotated-token" : current.csrf_token,
    });
    const { app, calls } = appWith(
      response(current),
      response(denied, 403),
      response(fresh),
      response(success),
    );
    await app.session.ensure();
    const result = await app.session.mutate(() => app.endpoints.signOut());
    assert.equal(result.ok, rotated);
    assert.equal(calls.length, rotated ? 4 : 3);
    const changes = calls.filter(({ init }) => init.method === "POST");
    assert.deepEqual(
      changes.map(({ init }) =>
        new Headers(init.headers).get("x-ciproxy-csrf"),
      ),
      rotated
        ? ["fixture-csrf-current", "rotated-token"]
        : ["fixture-csrf-current"],
    );
  }
});

test("page visits refresh old access and a failed refresh keeps its visible error", async () => {
  const { app, calls, setNow } = appWith(
    response(session()),
    response(
      { code: "unavailable", message: "Try again." } satisfies Problem,
      503,
    ),
    response(session({ csrf_token: "new-token" })),
  );
  await app.session.ensure();
  await app.session.visit();
  assert.equal(calls.length, 1);
  setNow(Date.parse(fixtureTime) + sessionFreshFor);
  const failed = await app.session.visit();
  assert.equal(failed.status, "signed-in");
  assert.deepEqual(
    failed.status === "signed-in" ? failed.refreshFailure : null,
    {
      kind: "problem",
      status: 503,
      problem: { code: "unavailable", message: "Try again." },
    },
  );
  await app.session.refresh();
  assert.equal(app.session.csrf(), "new-token");
});

test("403 reads refresh the session so a lost role becomes visible", async () => {
  const next = deferred<Response>();
  const { app, calls } = appWith(response(session()), next.promise);
  await app.session.ensure();
  app.session.report({
    ok: false,
    failure: { kind: "problem", status: 403, problem: denied },
  });
  assert.equal(calls.length, 2);
  next.resolve(response(session({ state: "access_ended", workspaces: [] })));
  await app.session.refresh();
  const state = app.session.state.get();
  assert.equal(
    state.status === "signed-in" ? state.session.state : null,
    "access_ended",
  );
});
