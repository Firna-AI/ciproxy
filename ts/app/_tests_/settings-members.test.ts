import assert from "node:assert/strict";
import test from "node:test";

import { membersModel, roleStep } from "../src/view/settings-members.ts";
import {
  repositoriesHref,
  settingsRoute,
  tabItems,
} from "../src/view/settings-tabs.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { member, membersView } from "./settings-support.ts";

test("a person with no role is a request, and the signed-in person has no Remove", () => {
  const model = membersModel(
    membersView([
      member("101", "mira", "admin", "2026-09-01T09:00:00Z"),
      member("105", "rowan", null, "2026-11-18T10:00:00Z"),
      member("103", "nora", "billing", "2026-10-01T23:30:00Z"),
      member("106", null, null, "2026-11-19T08:00:00Z"),
    ]),
    "mira",
    "UTC",
  );
  assert.deepEqual(model, {
    requests: [
      { id: "105", name: "rowan", asked: "Asked on 18 November" },
      { id: "106", name: "GitHub user 106", asked: "Asked on 19 November" },
    ],
    members: [
      {
        id: "101",
        name: "mira",
        you: true,
        role: "admin",
        since: "1 September",
      },
      {
        id: "103",
        name: "nora",
        you: false,
        role: "billing",
        since: "1 October",
      },
    ],
  });
});

test("a pick of Admin asks for the trust dialog, and other picks save at once", () => {
  assert.equal(roleStep("viewer", "admin"), "confirm-admin");
  assert.equal(roleStep(null, "admin"), "confirm-admin");
  assert.equal(roleStep("admin", "billing"), "save");
  assert.equal(roleStep(null, "viewer"), "save");
  assert.equal(roleStep("billing", "billing"), "none");
});

test("the Members tab counts the open requests in its name", () => {
  const names = (count: number | null) =>
    tabItems(workspaceId, "labels", count).map((item) => [
      item.label,
      item.count,
      item.accessibilityLabel,
      item.active,
    ]);
  assert.deepEqual(names(2), [
    ["Labels", null, "Labels", true],
    ["Overflow", null, "Overflow", false],
    ["Organizations", null, "Organizations", false],
    ["Members", 2, "Members, 2 open requests", false],
  ]);
  assert.deepEqual(names(1)[3], [
    "Members",
    1,
    "Members, 1 open request",
    false,
  ]);
  assert.deepEqual(names(0)[3], ["Members", null, "Members", false]);
});

test("the query picks the tab, and an organization opens its repositories", () => {
  assert.deepEqual(settingsRoute({}), { kind: "tab", tab: "labels" });
  assert.deepEqual(settingsRoute({ tab: "unknown" }), {
    kind: "tab",
    tab: "labels",
  });
  assert.deepEqual(settingsRoute({ tab: "members", organization: "x" }), {
    kind: "tab",
    tab: "members",
  });
  const href = repositoriesHref(workspaceId, "org 1");
  assert.equal(
    href,
    `/settings?workspace=${workspaceId}&tab=organizations&organization=org%201`,
  );
  const query = new URLSearchParams(href.split("?")[1]);
  assert.deepEqual(
    settingsRoute({
      tab: query.get("tab") ?? undefined,
      organization: query.get("organization") ?? undefined,
    }),
    { kind: "repositories", organizationId: "org 1" },
  );
});
