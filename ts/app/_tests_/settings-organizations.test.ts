import assert from "node:assert/strict";
import test from "node:test";

import { inactiveBanner } from "../src/view/settings-organization-banner.ts";
import {
  addedByText,
  organizationFailure,
  organizationRows,
} from "../src/view/settings-organizations.ts";
import { activation, workspaceId } from "../scripts/fixtures-core.ts";
import { held, linked, removed } from "./settings-support.ts";

test("the last linked organization has no Take out, and a removed one does not count", () => {
  const actions = (rows: ReturnType<typeof organizationRows>) =>
    rows.map((row) => [row.login, row.takeOut, row.repositories]);
  assert.deepEqual(
    actions(
      organizationRows([
        linked(1, "mokly-ai"),
        removed(2, "mokly-labs", "2026-12-20T10:00:00Z"),
      ]),
    ),
    [
      ["mokly-ai", false, true],
      ["mokly-labs", false, false],
    ],
  );
  assert.deepEqual(
    actions(
      organizationRows([
        linked(1, "mokly-ai"),
        linked(2, "mokly-labs", { state: "suspended" }),
        held(3, "mokly-studio", ["admin_approval"], workspaceId),
      ]),
    ),
    [
      ["mokly-ai", true, true],
      ["mokly-labs", true, true],
      ["mokly-studio", true, true],
    ],
  );
});

test("a removed organization shows the deadline of a new install that keeps its history", () => {
  const [row] = organizationRows(
    [removed(2, "mokly-labs", "2026-12-20T10:00:00Z")],
    "UTC",
  );
  assert.deepEqual(row, {
    id: "40000000-0000-4000-8000-000000000002",
    login: "mokly-labs",
    state: "removed",
    status: {
      text: "App removed",
      tone: "idle",
      lines: [
        "Install it again before 20 December at 10:00 to keep its job history",
      ],
    },
    access: "No repository access",
    chooseTeam: null,
    turnOn: false,
    repositories: false,
    takeOut: false,
    installAgain: true,
  });
});

test("an inactive row names each hold and offers the action that clears it", () => {
  const [plan, admin] = organizationRows([
    held(3, "mokly-studio", ["billing_upgrade", "admin_approval"], workspaceId),
    held(4, "mokly-lab-2", ["admin_approval"], workspaceId),
  ]);
  assert.deepEqual(plan?.status, {
    text: "Inactive",
    tone: "warn",
    lines: [
      "Free includes 1 organization",
      "The person who added it is no longer an Admin",
    ],
  });
  assert.deepEqual(
    [plan?.chooseTeam, plan?.turnOn],
    [`/billing?workspace=${workspaceId}`, false],
  );
  assert.deepEqual(
    [admin?.status?.lines, admin?.chooseTeam, admin?.turnOn],
    [["The person who added it is no longer an Admin"], null, true],
  );
  const [suspended] = organizationRows([
    linked(2, "mokly-labs", { state: "suspended" }),
  ]);
  assert.deepEqual(suspended?.status, {
    text: "App suspended",
    tone: "warn",
    lines: [],
  });
});

test("the repository access of a row reads the policy of its organization", () => {
  const rows = organizationRows([
    linked(1, "mokly-ai", { access: { mode: "all_private" } }),
    linked(2, "mokly-labs", {
      access: { mode: "selected", ids: ["1", "2", "3"] },
    }),
    linked(3, "mokly-one", { access: { mode: "selected", ids: ["1"] } }),
  ]);
  assert.deepEqual(
    rows.map((row) => row.access),
    [
      "All private repositories",
      "3 selected repositories",
      "1 selected repository",
    ],
  );
});

test("the banner names the inactive organization, its reasons, and the ones that keep running", () => {
  const banner = inactiveBanner([
    linked(1, "mokly-ai"),
    linked(2, "mokly-labs"),
    linked(5, "mokly-new", {
      activation: activation({ admitted: false, preparing: true }),
    }),
    removed(6, "mokly-old", "2026-12-20T10:00:00Z"),
    held(3, "mokly-studio", ["billing_upgrade", "admin_approval"], workspaceId),
  ]);
  assert.deepEqual(banner, {
    title: "mokly-studio is inactive",
    body: "Free includes 1 organization, and the person who added mokly-studio is no longer an Admin. It starts its jobs on Team, after an Admin turns it on. Jobs keep running in mokly-ai and mokly-labs.",
    chooseTeam: `/billing?workspace=${workspaceId}`,
    turnOn: null,
  });
  assert.deepEqual(
    inactiveBanner([
      linked(1, "mokly-ai"),
      held(3, "mokly-studio", ["admin_approval"], workspaceId),
    ]),
    {
      title: "mokly-studio is inactive",
      body: "The person who added mokly-studio is no longer an Admin. It starts its jobs after an Admin turns it on. Jobs keep running in mokly-ai.",
      chooseTeam: null,
      turnOn: {
        id: "40000000-0000-4000-8000-000000000003",
        login: "mokly-studio",
      },
    },
  );
  assert.equal(inactiveBanner([linked(1, "mokly-ai")]), null);
});

test("Turn on names the person who added the organization by the API's ID", () => {
  const organization = held(3, "mokly-studio", ["admin_approval"], workspaceId);
  assert.equal(
    addedByText(organization),
    "GitHub user 4783921 added mokly-studio and is no longer an Admin.",
  );
  assert.equal(
    addedByText({
      ...organization,
      activation: { ...organization.activation, added_by: null },
    }),
    "The person who added mokly-studio is no longer an Admin.",
  );
});

test("a refused take-out of the last organization says why", () => {
  assert.deepEqual(
    organizationFailure(
      {
        kind: "problem",
        status: 409,
        problem: { code: "conflict", message: "Conflict." },
      },
      "take-out",
    ),
    {
      title: "Unable to take the organization out",
      body: "The last organization of a workspace stays in it.",
    },
  );
});
