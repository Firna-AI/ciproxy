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
  const organizations = [
    { id: "40000000-0000-4000-8000-000000000001", login: "mokly-ai" },
    { id: "40000000-0000-4000-8000-000000000002", login: "mokly-labs" },
  ];
  const model = membersModel(
    membersView([
      member("101", "mira", "admin", "2026-09-01T09:00:00Z", {
        name: "Mira Chen",
      }),
      member("105", "rowan", null, "2026-11-18T10:00:00Z", {
        name: "Rowan Hale",
        request_organizations: organizations,
      }),
      member("103", "nora", "billing", "2026-10-01T23:30:00Z", {
        granted_at: "2026-10-03T08:00:00Z",
      }),
      member("104", "ellis", "viewer", "2026-10-02T09:00:00Z", {
        granted_at: null,
      }),
      member("106", null, null, "2026-11-19T08:00:00Z"),
    ]),
    "mira",
    "UTC",
  );
  assert.deepEqual(model, {
    requests: [
      {
        id: "105",
        name: "Rowan Hale",
        caption: "rowan",
        initials: "RH",
        asked: "Asked on 18 November\u00a0· member of mokly-ai and mokly-labs",
      },
      {
        id: "106",
        name: "GitHub user 106",
        caption: null,
        initials: "?",
        asked: "Asked on 19 November",
      },
    ],
    members: [
      {
        id: "101",
        name: "Mira Chen",
        caption: "mira",
        initials: "MC",
        you: true,
        role: "admin",
        since: "1 September",
      },
      {
        id: "103",
        name: "nora",
        caption: null,
        initials: "N",
        you: false,
        role: "billing",
        since: "3 October",
      },
      {
        id: "104",
        name: "ellis",
        caption: null,
        initials: "E",
        you: false,
        role: "viewer",
        since: null,
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
