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
    since: null,
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

test("an inactive row names each hold and the person who added it, and offers the action that clears it", () => {
  const anonymous = held(4, "mokly-lab-2", ["admin_approval"], workspaceId);
  const [plan, admin] = organizationRows([
    held(3, "mokly-studio", ["billing_upgrade", "admin_approval"], workspaceId),
    {
      ...anonymous,
      added_by: null,
      activation: { ...anonymous.activation, added_by: null },
    },
  ]);
  assert.deepEqual(plan?.status, {
    text: "Inactive",
    tone: "warn",
    lines: [
      "Free includes 1 organization",
      "Ana Souza, who added it, is no longer an Admin",
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

test("a row shows the day it joined and counts the private repositories of the latest snapshot", () => {
  const counts = (privateCount: string, allowed: string) => ({
    visible: "20",
    private: privateCount,
    allowed_private: allowed,
    observed_at: "2026-11-06T12:00:00Z",
  });
  const all = { mode: "all_private" } as const;
  const picked = { mode: "selected" as const, ids: ["1", "2", "3"] };
  const rows = organizationRows(
    [
      linked(1, "mokly-ai", { access: all, repositories: counts("14", "14") }),
      linked(2, "mokly-labs", {
        access: picked,
        repositories: counts("5", "2"),
      }),
      linked(3, "mokly-one", {
        access: picked,
        repositories: counts("1", "1"),
      }),
      linked(4, "mokly-docs", { access: all, repositories: counts("0", "0") }),
      linked(5, "mokly-new", {
        access: all,
        repositories: null,
        joined_at: null,
      }),
      linked(6, "mokly-old", { access: picked, repositories: null }),
    ],
    "UTC",
  );
  assert.deepEqual(
    rows.map((row) => [row.since, row.access]),
    [
      ["Since 1 September", "All 14 private repositories"],
      ["Since 1 September", "2 of 5 private repositories"],
      ["Since 1 September", "1 of 1 private repository"],
      ["Since 1 September", "No private repositories"],
      [null, "All private repositories"],
      ["Since 1 September", "Selected private repositories"],
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
    body: "Free includes 1 organization, and Ana Souza, who added mokly-studio, is no longer an Admin. It starts its jobs on Team, after an Admin turns it on. Jobs keep running in mokly-ai and mokly-labs.",
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
      body: "Ana Souza, who added mokly-studio, is no longer an Admin. It starts its jobs after an Admin turns it on. Jobs keep running in mokly-ai.",
      chooseTeam: null,
      turnOn: {
        id: "40000000-0000-4000-8000-000000000003",
        login: "mokly-studio",
      },
    },
  );
  assert.equal(inactiveBanner([linked(1, "mokly-ai")]), null);
  const both = held(
    3,
    "mokly-studio",
    ["billing_upgrade", "admin_approval"],
    workspaceId,
  );
  assert.equal(
    inactiveBanner([
      {
        ...both,
        added_by: null,
        activation: { ...both.activation, added_by: null },
      },
    ])?.body,
    "Free includes 1 organization, and the person who added mokly-studio is no longer an Admin. It starts its jobs on Team, after an Admin turns it on.",
  );
});

test("Turn on names the person who added the organization, and the day it joined", () => {
  const organization = held(3, "mokly-studio", ["admin_approval"], workspaceId);
  const profile = organization.added_by ?? null;
  assert.ok(profile);
  const cases = [
    [
      organization,
      "Ana Souza added mokly-studio on 1 September and is no longer an Admin.",
    ],
    [
      { ...organization, added_by: { ...profile, name: null } },
      "anasouza added mokly-studio on 1 September and is no longer an Admin.",
    ],
    [
      { ...organization, added_by: null },
      "GitHub user 4783921 added mokly-studio on 1 September and is no longer an Admin.",
    ],
    [
      { ...organization, joined_at: null },
      "Ana Souza added mokly-studio and is no longer an Admin.",
    ],
    [
      {
        ...organization,
        added_by: null,
        activation: { ...organization.activation, added_by: null },
      },
      "The person who added mokly-studio is no longer an Admin.",
    ],
  ] as const;
  for (const [view, text] of cases)
    assert.equal(addedByText(view, "UTC"), text);
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
