import assert from "node:assert/strict";
import test from "node:test";

import type { Problem, Success } from "../src/api/schema.ts";
import { settingsServices } from "../src/state/settings-services.ts";
import { session, workspaceId } from "../scripts/fixtures-core.ts";
import { appWith, deferred, response } from "./support.ts";
import {
  label,
  labelInput,
  linked,
  member,
  membersView,
  settingsView,
} from "./settings-support.ts";

const ok = { ok: true } satisfies Success;
const busy = { code: "unavailable", message: "Try again." } satisfies Problem;
const base = `/api/workspaces/${workspaceId}`;

function requests(calls: { url: string; init: RequestInit }[]) {
  return calls.map((call) => [call.init.method, call.url, call.init.body]);
}

test("a rename saves the name, then reads the settings and the session again", async () => {
  const { app, calls } = appWith(
    response(ok),
    response(settingsView({ name: "Mokly" })),
    response(session()),
  );
  const store = settingsServices(app).settings(workspaceId);
  const result = await store.rename("Mokly");
  assert.equal(result.ok, true);
  assert.deepEqual(requests(calls), [
    ["PUT", `${base}/settings`, JSON.stringify({ name: "Mokly" })],
    ["GET", `${base}/settings`, undefined],
    ["GET", "/api/session", undefined],
  ]);
  const state = store.state.get();
  assert.equal(state.status === "ready" ? state.value.name : null, "Mokly");
});

test("a take-out and Turn on change one organization by its ID", async () => {
  const organizationId = "40000000-0000-4000-8000-000000000002";
  const { app, calls } = appWith(
    response(ok),
    response(settingsView()),
    response(session()),
    response(ok),
    response(settingsView()),
    response(session()),
  );
  const store = settingsServices(app).settings(workspaceId);
  await store.takeOut(organizationId);
  await store.turnOn(organizationId);
  assert.deepEqual(
    requests(calls).filter(([method]) => method !== "GET"),
    [
      ["DELETE", `${base}/organizations/${organizationId}`, undefined],
      ["POST", `${base}/organizations/${organizationId}/enable`, undefined],
    ],
  );
});

test("labels and repository access save on their routes, and a preview changes nothing", async () => {
  const input = labelInput({ name: "load-tests" });
  const selection = { mode: "selected" as const, ids: ["7001"] };
  const { app, calls } = appWith(
    response(label("new", { name: "load-tests" })),
    response(settingsView()),
    response(label("old")),
    response(settingsView()),
    response(ok),
    response(settingsView()),
    response({ ...label("x").fit }),
    response(ok),
    response(settingsView()),
  );
  const store = settingsServices(app).settings(workspaceId);
  await store.saveLabel(null, input);
  await store.saveLabel("old", input);
  await store.deleteLabel("old");
  const preview = await store.previewLabel(input);
  await store.saveRepositories("org-1", selection);
  assert.equal(preview.ok, true);
  assert.deepEqual(requests(calls), [
    ["POST", `${base}/labels`, JSON.stringify(input)],
    ["GET", `${base}/settings`, undefined],
    ["PUT", `${base}/labels/old`, JSON.stringify(input)],
    ["GET", `${base}/settings`, undefined],
    ["DELETE", `${base}/labels/old`, undefined],
    ["GET", `${base}/settings`, undefined],
    ["POST", `${base}/labels/preview`, JSON.stringify(input)],
    [
      "PUT",
      `${base}/organizations/org-1/repositories`,
      JSON.stringify(selection),
    ],
    ["GET", `${base}/settings`, undefined],
  ]);
});

test("a failed change reads nothing again and returns its failure", async () => {
  const { app, calls } = appWith(response(busy, 503));
  const store = settingsServices(app).settings(workspaceId);
  const result = await store.rename("Mokly");
  assert.deepEqual(result, {
    ok: false,
    failure: { kind: "problem", status: 503, problem: busy },
  });
  assert.equal(calls.length, 1);
});

test("an older settings read cannot replace a newer one", async () => {
  const older = deferred<Response>();
  const { app } = appWith(
    older.promise,
    response(settingsView({ organizations: [linked(1, "mokly-ai")] })),
  );
  const store = settingsServices(app).settings(workspaceId);
  const first = store.load();
  await store.load();
  older.resolve(
    response(
      settingsView({
        organizations: [linked(1, "mokly-ai"), linked(2, "mokly-labs")],
      }),
    ),
  );
  await first;
  const state = store.state.get();
  assert.deepEqual(
    state.status === "ready"
      ? state.value.organizations.map((item) => item.login)
      : null,
    ["mokly-ai"],
  );
});

test("a failed refresh keeps the last good settings and shows the failure", async () => {
  const { app } = appWith(response(settingsView()), response(busy, 503));
  const store = settingsServices(app).settings(workspaceId);
  await store.load();
  await store.load();
  const state = store.state.get();
  assert.deepEqual(
    state.status === "ready" ? [state.value.name, state.refreshFailure] : null,
    ["Northstar", { kind: "problem", status: 503, problem: busy }],
  );
});

test("a role change saves the role, then reads the members, the settings, and the session", async () => {
  const { app, calls } = appWith(
    response(ok),
    response(membersView([member("104", "ellis", "admin")])),
    response(settingsView()),
    response(session()),
    response(ok),
    response(membersView([])),
    response(settingsView()),
    response(session()),
  );
  const store = settingsServices(app).members(workspaceId);
  await store.saveRole("104", "admin");
  await store.remove("104");
  assert.deepEqual(requests(calls), [
    ["PUT", `${base}/members/104`, JSON.stringify({ role: "admin" })],
    ["GET", `${base}/members`, undefined],
    ["GET", `${base}/settings`, undefined],
    ["GET", "/api/session", undefined],
    ["DELETE", `${base}/members/104`, undefined],
    ["GET", `${base}/members`, undefined],
    ["GET", `${base}/settings`, undefined],
    ["GET", "/api/session", undefined],
  ]);
});

test("an older members read cannot bring back a removed person", async () => {
  const older = deferred<Response>();
  const { app } = appWith(
    older.promise,
    response(membersView([member("101", "mira", "admin")])),
  );
  const store = settingsServices(app).members(workspaceId);
  const first = store.load();
  await store.load();
  older.resolve(
    response(
      membersView([
        member("101", "mira", "admin"),
        member("104", "ellis", "viewer"),
      ]),
    ),
  );
  await first;
  const state = store.state.get();
  assert.deepEqual(
    state.status === "ready"
      ? state.value.members.map((item) => item.login)
      : null,
    ["mira"],
  );
});
