import assert from "node:assert/strict";
import test from "node:test";

import type { ApiFailure } from "../src/api/client.ts";
import type { WorkspaceRole, WorkspaceView } from "../src/api/schema.ts";
import { sessionLanding, workspaceLanding } from "../src/routing/landing.ts";
import { workspaceAccess } from "../src/routing/workspace.ts";
import { workspacePages } from "../src/routing/pages.ts";
import { claimFailure } from "../src/view/claim.ts";
import {
  appRemovedPage,
  checkAgainTarget,
  installPage,
  noRolePage,
  personalAccountPage,
  signInPage,
} from "../src/view/access.ts";
import {
  accountMenu,
  navItems,
  organizationBanner,
  workspaceMenu,
} from "../src/view/shell.ts";
import {
  otherWorkspaceId,
  session,
  workspace,
  workspaceId,
} from "../scripts/fixtures-core.ts";
import { setup } from "../scripts/fixtures-setup.ts";

test("every private route gates ended access, removed App, and no role", () => {
  const noRole = {
    admins: ["mira"],
    can_claim_admin: false,
    id: "opaque-id",
    name: "Hidden",
    organization_limit_exceeded: false,
  } satisfies WorkspaceView;
  for (const page of workspacePages) {
    assert.deepEqual(
      workspaceAccess(session({ state: "app_removed" }), workspaceId, page),
      { kind: "redirect", href: "/" },
    );
    assert.deepEqual(
      workspaceAccess(session({ state: "access_ended" }), workspaceId, page),
      { kind: "redirect", href: "/" },
    );
    assert.deepEqual(
      workspaceAccess(session({ workspaces: [noRole] }), noRole.id, page),
      { kind: "no-role", workspace: noRole },
    );
  }
  assert.deepEqual(workspaceAccess(session(), "unknown", "jobs"), {
    kind: "redirect",
    href: "/",
  });
  assert.deepEqual(
    workspaceAccess(
      session({ workspaces: [workspace({ role: "viewer" })] }),
      workspaceId,
      "settings",
    ),
    { kind: "redirect", href: `/jobs?workspace=${workspaceId}` },
  );
});

test("landing obeys server access and setup outcomes without choosing an organization", () => {
  const context = { installStarted: false, preferredWorkspaceId: null };
  assert.deepEqual(sessionLanding(session({ state: "app_removed" }), context), {
    href: "/app-removed",
  });
  assert.deepEqual(
    sessionLanding(session({ state: "access_ended" }), context),
    { href: "/access-ended" },
  );
  assert.deepEqual(
    sessionLanding(
      session({ state: "install_required", workspaces: [] }),
      context,
    ),
    { href: "/install" },
  );
  assert.deepEqual(
    sessionLanding(
      session({
        state: "install_required",
        setup: {
          kind: "organizations_only",
          installation_id: "personal",
          remove_url: "https://github.com/settings/installations/9002",
        },
      }),
      context,
    ),
    { href: "/personal-account" },
  );
  const chosen = workspace({ id: "opaque / workspace" });
  assert.deepEqual(
    sessionLanding(session({ workspaces: [workspace(), chosen] }), {
      ...context,
      preferredWorkspaceId: chosen.id,
    }),
    { workspace: chosen },
  );
  assert.equal(
    workspaceLanding(chosen, setup()),
    "/setup?workspace=opaque%20%2F%20workspace",
  );
  assert.equal(
    workspaceLanding(workspace({ role: "billing" }), setup()),
    `/jobs?workspace=${workspaceId}`,
  );
});

test("shell permissions follow roles while workspace switching includes no-role workspaces", () => {
  const roles: WorkspaceRole[] = ["admin", "billing", "viewer"];
  const data = session({
    workspaces: [
      workspace(),
      workspace({ id: otherWorkspaceId, name: "Orbit", role: null }),
    ],
  });
  for (const role of roles) {
    const current = workspace({
      role,
      can_claim_admin: role !== "admin",
      organization_limit_exceeded: true,
    });
    const navigation = navItems(workspaceId, role, {
      done: 1,
      total: 3,
      complete: false,
    });
    assert.equal(
      navigation.some((item) => item.key === "settings"),
      role === "admin",
    );
    assert.equal(
      navigation.some((item) => item.key === "setup"),
      role === "admin",
    );
    assert.equal(
      workspaceMenu(data, current).canAddOrganization,
      role === "admin",
    );
    assert.equal(
      accountMenu(data, current, role).canClaim,
      current.can_claim_admin,
    );
    assert.deepEqual(
      organizationBanner(current, role)?.action,
      role === "viewer"
        ? { ask: "Ask an Admin or a Billing person" }
        : {
            label: "Plan and usage",
            href: `/billing?workspace=${workspaceId}`,
          },
    );
  }
  assert.deepEqual(
    workspaceMenu(data, workspace()).choices.map((choice) => [
      choice.name,
      choice.detail,
    ]),
    [
      ["Northstar", "Admin"],
      ["Orbit", "No role yet"],
    ],
  );
});

test("access actions use server URLs and the exact claim permission", () => {
  const data = session({
    state: "install_required",
    install_url: "/install",
    reauthorization_url: "/api/auth/github?proof=fixture",
  });
  assert.deepEqual(checkAgainTarget(data), {
    leave: "/api/auth/github?proof=fixture",
  });
  assert.deepEqual(checkAgainTarget(session()), { refresh: true });
  assert.equal(installPage(data).actions[0]?.kind, "install");
  const remove = personalAccountPage(
    data,
    "https://github.com/settings/installations/exact",
  ).actions.find((action) => action.kind === "leave" && action.newTab);
  assert.equal(
    remove?.kind === "leave" ? remove.href : null,
    "https://github.com/settings/installations/exact",
  );
  assert.equal(
    noRolePage(
      data,
      workspace({ role: null, can_claim_admin: false }),
    ).actions.some((action) => action.kind === "claim-admin"),
    false,
  );
  assert.deepEqual(
    noRolePage(data, workspace({ role: null, can_claim_admin: true }))
      .actions[0],
    {
      kind: "claim-admin",
      label: "Claim the Admin role",
      tone: "primary",
      workspaceId,
    },
  );
  for (const reason of [null, "github", "ended"] as const) {
    const action = signInPage(reason).actions[0];
    assert.equal(
      action?.kind === "leave" ? action.href : null,
      "/api/auth/github",
    );
  }
  const removed = appRemovedPage(
    data,
    {
      github_id: "8001",
      login: "northstar-labs",
      delete_after: "2026-12-06T12:00:00Z",
    },
    "UTC",
  );
  assert.match(removed.title, /northstar-labs/);
  assert.match(removed.notice?.title ?? "", /6 December/);
});

test("claim failures give recovery without inventing an ownership cause", () => {
  assert.deepEqual(claimFailure({ kind: "network" }), {
    title: "Unable to claim the Admin role",
    body: "Check the connection, then try again.",
  });
  const refused: ApiFailure[] = [
    {
      kind: "problem",
      status: 403,
      problem: { code: "access_denied", message: "The CSRF token changed." },
    },
    {
      kind: "problem",
      status: 503,
      problem: { code: "unavailable", message: "GitHub did not answer." },
    },
  ];
  for (const failure of refused) {
    assert.deepEqual(claimFailure(failure), {
      title: "Unable to claim the Admin role",
      body: "ciproxy could not complete the claim. Try again.",
    });
  }
});
