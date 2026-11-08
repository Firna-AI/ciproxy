import assert from "node:assert/strict";

import { workspaceHomeHref } from "../src/routing/pages.ts";
import { storageKeys } from "../src/state/storage.ts";
import type { Page } from "./browser-driver.ts";
import {
  control,
  facts,
  viewportControls,
  visible,
} from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import { otherWorkspaceId, workspaceId } from "./fixtures-core.ts";

const rememberedViewer = `if (sessionStorage.getItem("ciproxy.fixture-seeded") !== "1") {
  localStorage.setItem(${JSON.stringify(storageKeys.lastWorkspace)}, ${JSON.stringify(otherWorkspaceId)});
  sessionStorage.setItem("ciproxy.fixture-seeded", "1");
}`;

async function explicitWorkspaceStays(
  page: Page,
  origin: string,
): Promise<void> {
  await page.goto(origin + workspaceHomeHref(otherWorkspaceId), {
    waitUntil: "domcontentloaded",
  });
  await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
  await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
  await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
  assert.equal(
    new URL(page.url()).searchParams.get("workspace"),
    otherWorkspaceId,
    "A handled continuation must preserve the selected workspace on reload",
  );
}

async function lateSignIn(
  page: Page,
  origin: string,
  width: number,
  journey: "first" | "add",
): Promise<void> {
  await control(origin, {
    scenario: journey === "first" ? "late-first-wait" : "late-add-wait",
  });
  await page.goto(
    origin + (journey === "first" ? "/install" : "/add-organization"),
    { waitUntil: "domcontentloaded" },
  );
  await visible(
    page.getByText("Waiting for an owner of northstar-labs", { exact: true }),
  );
  await viewportControls(page, width);
  if (journey === "first") {
    await page
      .getByRole("button", { name: "Check again", exact: true })
      .click();
  } else {
    await page.goto(origin + "/sign-in", { waitUntil: "domcontentloaded" });
    await page
      .getByRole("link", { name: "Sign in with GitHub", exact: true })
      .click();
  }
  await page.waitForURL(/\/(?:jobs|setup)\?workspace=/);
  const destination = new URL(page.url());
  assert.equal(
    destination.searchParams.get("workspace"),
    workspaceId,
    "Plain sign-in must open the exact newly resolved claim's workspace",
  );
  assert.equal(destination.pathname, journey === "first" ? "/setup" : "/jobs");
  await visible(
    page.getByRole("heading", {
      name: journey === "first" ? "Set up ciproxy" : "Jobs",
      exact: true,
    }),
  );
  const observed = await facts(origin);
  const authorization = observed.findIndex(
    (fact) => fact.path === "/api/auth/github" && fact.kind === "navigation",
  );
  assert.notEqual(authorization, -1, "The journey must use top-level sign-in");
  assert.ok(
    observed.some(
      (fact, index) =>
        index > authorization && fact.path === "/" && fact.kind === "document",
    ),
    "The authorization must return a new landing document",
  );
  await explicitWorkspaceStays(page, origin);
}

async function multipleClaims(
  page: Page,
  origin: string,
  width: number,
): Promise<void> {
  await control(origin, { scenario: "late-multiple-completed" });
  await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
  await page.waitForURL(/\/(?:jobs|setup|add-organization)(?:\?|$)/);
  assert.equal(
    new URL(page.url()).pathname,
    "/add-organization",
    "Several resolved destinations must not choose one by claim order",
  );
  for (const [organization, name, id] of [
    ["northstar-labs", "Northstar", workspaceId],
    ["orbit-labs", "Orbit", otherWorkspaceId],
  ] as const) {
    await visible(
      page.getByRole("heading", {
        name: `${organization} joined ${name}`,
        exact: true,
      }),
    );
    const open = page.getByRole("link", { name: `Open ${name}`, exact: true });
    await visible(open);
    assert.equal(await open.getAttribute("href"), workspaceHomeHref(id));
  }
  await viewportControls(page, width);
  await page.getByRole("link", { name: "Open Orbit", exact: true }).click();
  await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
  assert.equal(
    new URL(page.url()).searchParams.get("workspace"),
    otherWorkspaceId,
  );
  await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
  await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
  assert.equal(
    new URL(page.url()).searchParams.get("workspace"),
    otherWorkspaceId,
  );
}

export function createLateApprovalRuns(origin: string): readonly SmokeRun[] {
  return [
    ...(["first", "add"] as const).map((journey): SmokeRun => ({
      name: `late-${journey}-sign-in`,
      init: rememberedViewer,
      check: (page, _context, width) =>
        lateSignIn(page, origin, width, journey),
    })),
    {
      name: "late-multiple-claims",
      init: rememberedViewer,
      check: (page, _context, width) => multipleClaims(page, origin, width),
    },
  ];
}
