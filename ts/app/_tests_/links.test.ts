import assert from "node:assert/strict";
import test from "node:test";

import type { Problem } from "../src/api/schema.ts";
import { createLinksStore } from "../src/state/links.ts";
import { linkSection } from "../src/view/add-organization.ts";
import { requestSection } from "../src/view/link-notes.ts";
import {
  activation,
  claimId,
  link,
  otherClaimId,
  otherWorkspaceId,
  session,
  workspaceId,
} from "../scripts/fixtures-core.ts";
import { appWith, deferred, response, settled } from "./support.ts";

function linksWith(...replies: Parameters<typeof appWith>) {
  const context = appWith(...replies);
  const store = createLinksStore({
    endpoints: context.app.endpoints,
    session: context.app.session,
    timers: context.schedule.timers,
    visibility: context.schedule.visibility,
  });
  return { ...context, store, stop: store.watch() };
}

test("workspace offers start unselected and each claim keeps only a still-offered pick", () => {
  const { store, stop } = linksWith();
  const one = link();
  const two = link({ id: otherClaimId, organization: "orbit-labs" });
  store.seed([one, two]);
  assert.deepEqual(
    store.state.get().entries.map((entry) => entry.picked),
    [null, null],
  );
  const view = linkSection(session(), store.state.get().entries[0]!);
  assert.deepEqual(
    view.choices?.map((choice) => choice.checked),
    [false, false],
  );
  assert.equal(view.submit?.enabled, false);
  assert.match(
    view.choices?.find((choice) => choice.id === workspaceId)?.note ?? "",
    /inactive/,
  );
  store.pick(claimId, workspaceId);
  store.pick(otherClaimId, otherWorkspaceId);
  store.seed([one, { ...two, workspaces: [two.workspaces[0]!] }]);
  assert.deepEqual(
    store.state.get().entries.map((entry) => entry.picked),
    [workspaceId, null],
  );
  store.seed([one, two]);
  assert.deepEqual(
    store.state.get().entries.map((entry) => entry.picked),
    [workspaceId, null],
  );
  stop();
});

test("a selected claim submits its exact destination once while another claim stays independent", async () => {
  const pending = deferred<Response>();
  const { store, calls, stop } = linksWith(pending.promise);
  store.seed([link(), link({ id: otherClaimId })]);
  await store.choose(claimId);
  assert.equal(calls.length, 0);
  store.pick(claimId, otherWorkspaceId);
  store.pick(otherClaimId, workspaceId);
  const choosing = store.choose(claimId);
  await store.choose(claimId);
  assert.equal(calls.length, 1);
  assert.equal(calls[0]?.init.body, `{"workspace_id":"${otherWorkspaceId}"}`);
  assert.deepEqual(
    store.state.get().entries.map((entry) => [entry.choosing, entry.picked]),
    [
      [true, otherWorkspaceId],
      [false, workspaceId],
    ],
  );
  pending.resolve(
    response(
      link({
        state: "linked",
        workspace_id: otherWorkspaceId,
        activation: activation(),
      }),
    ),
  );
  await choosing;
  assert.equal(
    store.state.get().entries[0]?.link.workspace_id,
    otherWorkspaceId,
  );
  assert.equal(store.state.get().entries[1]?.picked, workspaceId);
  stop();
});

test("a stale destination refreshes that claim and preserves other claims", async () => {
  const conflict = {
    code: "conflict",
    message: "Choose from the current workspaces.",
  } satisfies Problem;
  const updated = link({
    workspaces: [
      { id: otherWorkspaceId, name: "Orbit", requires_upgrade: false },
    ],
  });
  const { store, calls, stop } = linksWith(
    response(conflict, 409),
    response(updated),
  );
  store.seed([link(), link({ id: otherClaimId })]);
  store.pick(claimId, workspaceId);
  store.pick(otherClaimId, otherWorkspaceId);
  await store.choose(claimId);
  assert.deepEqual(
    calls.map((call) => call.url),
    [
      `/api/installation-links/${claimId}/workspace`,
      `/api/installation-links/${claimId}`,
    ],
  );
  assert.equal(store.state.get().entries[0]?.picked, null);
  assert.equal(store.state.get().entries[0]?.failure?.kind, "problem");
  assert.equal(store.state.get().entries[1]?.picked, otherWorkspaceId);
  stop();
});

test("link poll failures stay visible and a later moving claim resumes polling", async () => {
  const { store, schedule, stop } = linksWith(
    response(
      {
        code: "unavailable",
        message: "GitHub is unavailable.",
      } satisfies Problem,
      503,
    ),
    response(link({ state: "linked", workspace_id: workspaceId })),
    response(
      link({
        id: otherClaimId,
        state: "linked",
        workspace_id: otherWorkspaceId,
      }),
    ),
  );
  store.seed([link({ state: "linking" })]);
  store.watch();
  schedule.fire();
  await settled();
  assert.equal(store.state.get().entries[0]?.failure?.kind, "problem");
  assert.equal(schedule.pending(), 1);
  schedule.fire();
  await settled();
  assert.equal(store.state.get().entries[0]?.failure, null);
  assert.equal(schedule.pending(), 0);
  store.seed([link({ id: otherClaimId, state: "linking" })]);
  assert.equal(schedule.pending(), 1, "A new moving claim must resume polling");
  schedule.fire();
  await settled();
  assert.equal(store.state.get().entries[1]?.link.state, "linked");
  stop();
});

test("link recovery and inactive actions come only from the server state", () => {
  const entry = {
    link: link({ state: "reauthorization_required" }),
    picked: otherWorkspaceId,
    choosing: false,
    failure: null,
  };
  const recovery = linkSection(session(), entry);
  assert.equal(recovery.choices, null);
  assert.equal(
    recovery.actions[0]?.kind === "leave" ? recovery.actions[0].href : null,
    "/api/auth/github?fixture=claim-reauthorize",
  );
  const inactive = linkSection(session(), {
    ...entry,
    link: link({
      state: "linked_inactive",
      workspace_id: workspaceId,
      activation: activation({
        admitted: false,
        reasons: ["billing_upgrade", "admin_approval"],
        upgrade: { tier: "team", workspace_id: otherWorkspaceId },
        can_enable: false,
        ask_admin: true,
      }),
    }),
  });
  assert.equal(inactive.reasons.length, 2);
  assert.deepEqual(
    inactive.actions
      .filter((action) => action.kind === "go")
      .map((action) => action.href),
    [`/billing?workspace=${otherWorkspaceId}`, `/?workspace=${workspaceId}`],
  );
  const request = requestSection(
    session({
      pending_install_requests: [
        {
          id: "91",
          github_id: "8001",
          organization: "orbit-labs",
          created_at: "2026-11-06T12:00:00Z",
        },
      ],
    }),
  );
  assert.equal(request?.choices, null);
  assert.equal(request?.actions[0]?.kind, "check-again");
});
