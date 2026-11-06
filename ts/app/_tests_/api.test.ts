import assert from "node:assert/strict";
import test from "node:test";

import { createApiClient } from "../src/api/client.ts";
import { createEndpoints } from "../src/api/endpoints.ts";
import { parseSession, parseSuccess } from "../src/api/parse-session.ts";
import type { Problem, Success } from "../src/api/schema.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import { link, session } from "../scripts/fixtures-core.ts";
import { response, transportQueue } from "./support.ts";

test("API requests use same-origin credentials and the current CSRF token", async () => {
  let csrf = "old-token";
  const queue = transportQueue(
    response(session()),
    response({ ok: true } satisfies Success),
    response(link()),
  );
  const endpoints = createEndpoints(
    createApiClient({
      ...queue,
      csrf: () => csrf,
      gate: createBuildGate(),
    }),
  );
  await endpoints.session();
  csrf = "fresh-token";
  await endpoints.claimAdmin("opaque / workspace");
  await endpoints.chooseWorkspace("claim/one", "exact-destination");
  assert.deepEqual(
    queue.calls.map(({ url, init }) => ({
      url,
      method: init.method,
      credentials: init.credentials,
      cache: init.cache,
      csrf: new Headers(init.headers).get("x-ciproxy-csrf"),
      accept: new Headers(init.headers).get("Accept"),
      type: new Headers(init.headers).get("Content-Type"),
      body: init.body,
    })),
    [
      {
        url: "/api/session",
        method: "GET",
        credentials: "same-origin",
        cache: "no-store",
        csrf: null,
        accept: "application/json",
        type: null,
        body: undefined,
      },
      {
        url: "/api/workspaces/opaque%20%2F%20workspace/claim-admin",
        method: "POST",
        credentials: "same-origin",
        cache: "no-store",
        csrf: "fresh-token",
        accept: "application/json",
        type: null,
        body: undefined,
      },
      {
        url: "/api/installation-links/claim%2Fone/workspace",
        method: "PUT",
        credentials: "same-origin",
        cache: "no-store",
        csrf: "fresh-token",
        accept: "application/json",
        type: "application/json",
        body: '{"workspace_id":"exact-destination"}',
      },
    ],
  );
});

test("API preserves typed HTTP failures and rejects malformed or unreachable responses", async () => {
  const denied = {
    code: "access_denied",
    message: "Access changed.",
  } satisfies Problem;
  const cases = [
    {
      reply: response(denied, 403),
      expected: { kind: "problem", status: 403, problem: denied },
    },
    {
      reply: response({ code: "unknown-code", message: "bad" }, 500),
      expected: { kind: "malformed", status: 500 },
    },
    {
      reply: new Response("<html>unavailable</html>", { status: 502 }),
      expected: { kind: "malformed", status: 502 },
    },
    {
      reply: response({ incomplete: true }),
      expected: { kind: "malformed", status: 200 },
    },
    { reply: new Error("offline"), expected: { kind: "network" } },
  ];
  for (const { reply, expected } of cases) {
    const client = createApiClient({
      ...transportQueue(reply),
      gate: createBuildGate(),
      csrf: () => null,
    });
    assert.deepEqual(await client.read("/session", parseSession), {
      ok: false,
      failure: expected,
    });
  }
});

test("error and mutation responses latch a build change despite a late old response", async () => {
  const gate = createBuildGate();
  const queue = transportQueue(
    response({ ok: true } satisfies Success),
    response(
      { code: "conflict", message: "Try again." } satisfies Problem,
      409,
      "fixture-build-b",
    ),
    response({ ok: true } satisfies Success),
  );
  const client = createApiClient({ ...queue, gate, csrf: () => "fixture" });
  await client.read("/session", parseSuccess);
  assert.equal(gate.changed(), false);
  await client.change("POST", "/auth/sign-out", parseSuccess);
  assert.equal(gate.changed(), true);
  await client.read("/session", parseSuccess);
  assert.equal(gate.changed(), true);
  assert.equal(gate.reloading(), false);
});

test("API dispatch reloads a changed page before a late history listener", async () => {
  let page = "/setup?workspace=opaque-id";
  const gate = createBuildGate(() => page);
  const queue = transportQueue(
    response(session()),
    response(session(), 200, "fixture-build-b"),
    response(session(), 200, "fixture-build-b"),
    response(session(), 200, "fixture-build-b"),
  );
  const reloads: { page: string; blocked: boolean; sent: number }[] = [];
  const endpoints = createEndpoints(
    createApiClient({
      ...queue,
      gate,
      csrf: () => null,
      reload: () => {
        reloads.push({
          page,
          blocked: gate.reloading(),
          sent: queue.calls.length,
        });
      },
    }),
  );
  await endpoints.session();
  await endpoints.session();
  assert.equal(gate.changed(), true);
  assert.equal(gate.reloading(), false);
  assert.deepEqual(reloads, []);

  // The current page can finish a read after it observes the changed build.
  assert.deepEqual(await endpoints.session(), { ok: true, value: session() });
  assert.equal(queue.calls.length, 3);

  // The router changes this identity before any history listener runs.
  page = "/jobs?workspace=opaque-id";
  // Reload deliberately abandons this promise. The guard must act now.
  void endpoints.session();
  assert.deepEqual(reloads, [
    { page: "/jobs?workspace=opaque-id", blocked: true, sent: 3 },
  ]);
  assert.equal(gate.reloading(), true);
  assert.equal(queue.calls.length, 3);
});

test("session parsing accepts an omitted role as no access", () => {
  const withoutRole = {
    admins: ["owner"],
    can_claim_admin: true,
    id: "opaque-id",
    name: "Private",
    organization_limit_exceeded: false,
  };
  const parsed = parseSession(session({ workspaces: [withoutRole] }));
  assert.equal(parsed?.workspaces[0]?.role, null);
});
