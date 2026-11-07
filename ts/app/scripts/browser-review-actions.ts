import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import type { SmokeCase } from "./browser-cases.ts";
import { control, facts, insideViewport, visible } from "./browser-fixture.ts";
import {
  longWorkspaceName,
  otherWorkspaceId,
  workspaceId,
} from "./fixtures-core.ts";

export async function exerciseReviewCase(
  page: Page,
  item: SmokeCase,
  origin: string,
  width: number,
): Promise<void> {
  if (item.name === "workspace-long") {
    await insideViewport(
      page.getByRole("button", { name: "Account of mira", exact: true }),
      width,
    );
    await insideViewport(
      page.getByRole("button", {
        name: `Workspace ${longWorkspaceName}. Switch workspace`,
        exact: true,
      }),
      width,
    );
  }
  if (
    [
      "root-install-completed",
      "root-install-viewer",
      "root-install-github",
      "root-install-github-empty",
    ].includes(item.name)
  ) {
    assert.equal(
      new URL(page.url()).searchParams.get("workspace"),
      workspaceId,
    );
  }
  if (
    item.name === "root-install-github" ||
    item.name === "root-install-github-empty"
  ) {
    await page.goto(`${origin}/?workspace=${otherWorkspaceId}`, {
      waitUntil: "domcontentloaded",
    });
    await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
    await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
    await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
    assert.equal(
      new URL(page.url()).searchParams.get("workspace"),
      otherWorkspaceId,
    );
  }
  if (
    item.name === "root-install-handled" ||
    item.name === "root-explicit-workspace"
  ) {
    assert.equal(
      new URL(page.url()).searchParams.get("workspace"),
      otherWorkspaceId,
    );
  }
  if (item.name === "root-explicit-workspace") {
    await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
    await visible(
      page.getByRole("heading", { name: "Set up ciproxy", exact: true }),
    );
    assert.equal(
      new URL(page.url()).searchParams.get("workspace"),
      workspaceId,
    );
  }
  if (item.name === "root-install-progress") {
    assert.equal(new URL(page.url()).pathname, "/add-organization");
    await control(origin, { scenario: "root-install-viewer", reset: false });
    await page.goto(origin + "/", { waitUntil: "domcontentloaded" });
    await visible(
      page.getByRole("heading", { name: "Set up ciproxy", exact: true }),
    );
    assert.equal(
      new URL(page.url()).searchParams.get("workspace"),
      workspaceId,
    );
  }
  if (
    item.name === "root-removed-old-claim" ||
    item.name === "root-removed-old-outcome"
  ) {
    assert.equal(new URL(page.url()).pathname, "/app-removed");
    assert.equal(
      await page
        .getByText("Unable to confirm the install on GitHub", { exact: true })
        .count(),
      0,
    );
  }
  if (
    item.name === "root-reinstall-open" ||
    item.name === "root-access-recovery"
  ) {
    assert.equal(new URL(page.url()).pathname, "/add-organization");
    await visible(
      page.getByRole("link", { name: "Sign in with GitHub", exact: true }),
    );
  }
  if (item.name.startsWith("session-refresh-"))
    await failedSessionCheck(page, item, origin);
  if (
    item.name.startsWith("claim-failure-") ||
    item.name === "claim-network-failure"
  )
    await failedClaim(page, item);
}

async function failedSessionCheck(
  page: Page,
  item: SmokeCase,
  origin: string,
): Promise<void> {
  const workspacePage = item.name === "session-refresh-workspace";
  if (workspacePage) {
    await page.clock.runFor(2001);
    await page.getByRole("link", { name: "Machines", exact: true }).click();
  } else {
    await page
      .getByRole("button", { name: "Check again", exact: true })
      .click();
  }
  const failure = page.getByText("Unable to check your access", {
    exact: true,
  });
  await visible(failure);
  const title = workspacePage
    ? "Machines"
    : "Waiting for an Admin of Northstar";
  await visible(page.getByRole("heading", { name: title, exact: true }));
  if (!workspacePage) await visible(page.getByText("sasha", { exact: true }));
  assert.deepEqual(
    (await facts(origin))
      .filter((fact) => fact.path === "/api/session")
      .map((fact) => fact.status),
    [200, 503],
  );
  await control(origin, {
    scenario: workspacePage ? "viewer" : "no-role",
    reset: false,
  });
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await failure.waitFor({ state: "hidden" });
  await visible(page.getByRole("heading", { name: title, exact: true }));
  assert.deepEqual(
    (await facts(origin))
      .filter((fact) => fact.path === "/api/session")
      .map((fact) => fact.status),
    [200, 503, 200],
  );
}

async function failedClaim(page: Page, item: SmokeCase): Promise<void> {
  if (item.name === "claim-failure-workspace") {
    await page
      .getByRole("button", { name: "Account of mira", exact: true })
      .click();
    await page
      .getByRole("menuitem", { name: "Claim the Admin role", exact: true })
      .click();
  } else {
    await page
      .getByRole("button", { name: "Claim the Admin role", exact: true })
      .click();
  }
  await visible(
    page.getByText("Unable to claim the Admin role", { exact: true }),
  );
  await visible(
    page.getByText(
      item.name === "claim-network-failure"
        ? "Check the connection, then try again."
        : "ciproxy could not complete the claim. Try again.",
      { exact: true },
    ),
  );
  assert.equal(
    await page.getByText(/GitHub must list you as an owner/).count(),
    0,
  );
}
