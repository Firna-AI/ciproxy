import assert from "node:assert/strict";
import test from "node:test";

import type { Problem, Success } from "../src/api/schema.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { installCommand } from "../scripts/fixtures-setup.ts";
import { appWith, deferred, response } from "./support.ts";

test("install command creation is explicit, deduplicated, and can be revoked", async () => {
  const issue = deferred<Response>();
  const { app, calls } = appWith(
    issue.promise,
    response({ ok: true } satisfies Success),
  );
  const store = app.installCommand(workspaceId);
  assert.deepEqual(store.state.get(), { status: "none" });
  assert.equal(calls.length, 0);
  const creating = store.create();
  await store.create();
  assert.equal(calls.length, 1);
  assert.deepEqual(store.state.get(), { status: "creating" });
  issue.resolve(response(installCommand()));
  await creating;
  await store.create();
  assert.equal(calls.length, 1);
  await store.revoke();
  assert.equal(calls[1]?.init.method, "DELETE");
  assert.equal(
    calls[1]?.url,
    `/api/workspaces/${workspaceId}/install-tokens/fixture-install-token`,
  );
  assert.deepEqual(store.state.get(), { status: "revoked" });
});

test("an expired command is replaced only on request at its exact expiry", async () => {
  const { app, calls, setNow } = appWith(
    response(installCommand()),
    response(
      installCommand({ id: "replacement", expires_at: "2026-11-06T14:00:00Z" }),
    ),
  );
  const store = app.installCommand(workspaceId);
  await store.create();
  setNow(Date.parse("2026-11-06T12:59:59.999Z"));
  await store.create();
  assert.equal(calls.length, 1);
  setNow(Date.parse("2026-11-06T13:00:00Z"));
  assert.equal(calls.length, 1);
  await store.create();
  const current = store.state.get();
  assert.equal(
    current.status === "open" ? current.command.id : null,
    "replacement",
  );
  assert.equal(calls.length, 2);
});

test("failed create and revoke remain visible and allow explicit recovery", async () => {
  const problem = {
    code: "unavailable",
    message: "Try again.",
  } satisfies Problem;
  const { app } = appWith(
    response(problem, 503),
    response(installCommand()),
    response(problem, 503),
    response({ ok: true } satisfies Success),
  );
  const store = app.installCommand(workspaceId);
  await store.create();
  assert.deepEqual(store.state.get(), {
    status: "failed",
    failure: { kind: "problem", status: 503, problem },
  });
  await store.create();
  await store.revoke();
  const failed = store.state.get();
  assert.equal(
    failed.status === "open" ? failed.command.id : null,
    "fixture-install-token",
  );
  assert.deepEqual(failed.status === "open" ? failed.revokeFailure : null, {
    kind: "problem",
    status: 503,
    problem,
  });
  await store.revoke();
  assert.deepEqual(store.state.get(), { status: "revoked" });
});
