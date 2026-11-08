import assert from "node:assert/strict";

import type { BrowserContext, Page } from "./browser-driver.ts";
import type { SmokeCase } from "./browser-cases.ts";
import { control, facts, keyboardMenu, visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { exerciseReviewCase } from "./browser-review-actions.ts";

async function pickWorkspace(page: Page): Promise<void> {
  const northstar = page.getByRole("radio", { name: /Northstar/ });
  const orbit = page.getByRole("radio", { name: /Orbit/ });
  await visible(northstar);
  assert.equal(await northstar.isChecked(), false);
  assert.equal(await orbit.isChecked(), false);
  assert.equal(
    await page
      .getByRole("button", { name: "Add northstar-labs", exact: true })
      .isEnabled(),
    false,
  );
  await northstar.click();
  assert.equal(await orbit.isChecked(), false);
  await page
    .getByRole("button", {
      name: "Add northstar-labs to Northstar",
      exact: true,
    })
    .click();
  await visible(
    page.getByRole("heading", {
      name: "northstar-labs joined Northstar, inactive",
      exact: true,
    }),
  );
}

export async function exerciseCase(
  page: Page,
  context: BrowserContext,
  item: SmokeCase,
  origin: string,
  width: number,
): Promise<void> {
  await exerciseReviewCase(page, item, origin, width);
  if (
    item.name.startsWith("links-inactive") ||
    item.name === "links-preparing"
  ) {
    await pickWorkspace(page);
    if (item.name === "links-preparing")
      await visible(page.getByText("Preparing its runners…", { exact: true }));
    if (item.name === "links-inactive-admin")
      await visible(
        page.getByRole("link", { name: "Turn on in Settings", exact: true }),
      );
    if (
      item.name === "links-inactive-plan" ||
      item.name === "links-inactive-both"
    )
      await visible(
        page.getByRole("link", { name: "Plan and usage", exact: true }),
      );
  }
  if (item.name === "links-picker") {
    await pickWorkspace(page);
    const observed = await facts(origin);
    assert.equal(
      observed.filter(
        (fact) => fact.method === "PUT" && fact.path.endsWith("/workspace"),
      ).length,
      1,
    );
  }
  if (item.name === "setup-connected") {
    await visible(page.getByTestId("install-command"));
    await control(origin, { scenario: "setup-connected", reset: false });
    await page.clock.runFor(5001);
    await visible(page.getByText("forge-01 connected", { exact: true }));
    await visible(page.getByRole("button", { name: "Continue", exact: true }));
    const observed = await facts(origin);
    assert.equal(
      observed.filter(
        (fact) =>
          fact.method === "POST" && fact.path.endsWith("/install-tokens"),
      ).length,
      1,
    );
  }
  if (item.name === "setup-success" || item.name === "setup-failure") {
    await visible(page.getByRole("link", { name: "Open Jobs", exact: true }));
    await visible(
      page.getByText(
        item.name === "setup-success"
          ? /passed in 6m 12s on your machine/
          : /failed on your machine/,
      ),
    );
  }
  if (item.name === "command-revoke-failure") {
    await page
      .getByRole("button", { name: "Revoke token", exact: true })
      .click();
    await visible(
      page.getByText("Unable to revoke the token", { exact: true }),
    );
  }
  if (item.name === "setup-waiting") {
    const before = await page.getByTestId("install-command").textContent();
    await context.grantPermissions(["clipboard-read", "clipboard-write"], {
      origin,
    });
    await page
      .getByRole("button", { name: "Copy the install command", exact: true })
      .click();
    await visible(
      page.getByRole("button", {
        name: "Copied the install command",
        exact: true,
      }),
    );
    await page
      .getByRole("button", { name: "Revoke token", exact: true })
      .click();
    await visible(page.getByText("Token revoked", { exact: true }));
    await page
      .getByRole("button", { name: "Create a new command", exact: true })
      .click();
    await visible(page.getByTestId("install-command"));
    assert.notEqual(
      await page.getByTestId("install-command").textContent(),
      before,
      "The replacement command must use different text",
    );
    await visible(
      page.getByRole("button", {
        name: "Copy the install command",
        exact: true,
      }),
    );
    assert.equal(
      await page
        .getByRole("button", {
          name: "Copied the install command",
          exact: true,
        })
        .count(),
      0,
    );
    await page
      .getByRole("button", { name: "Copy the install command", exact: true })
      .click();
    await visible(
      page.getByRole("button", {
        name: "Copied the install command",
        exact: true,
      }),
    );
  }
  if (item.name === "setup-available") {
    await context.grantPermissions(["clipboard-read", "clipboard-write"], {
      origin,
    });
    await page
      .getByRole("button", { name: "Copy the workflow change", exact: true })
      .click();
    await visible(
      page.getByRole("button", {
        name: "Copied the workflow change",
        exact: true,
      }),
    );
  }
  if (
    item.name === "setup-load-failure" ||
    item.name === "command-load-failure"
  ) {
    await control(origin, { scenario: "setup-waiting", reset: false });
    await page.getByRole("button", { name: "Try again", exact: true }).click();
    await visible(page.getByTestId("install-command"));
  }
  if (
    ["admin", "billing", "viewer", "billing-claim", "viewer-claim"].includes(
      item.name,
    )
  ) {
    await keyboardMenu(page, "Account of mira", "Sign out");
    await keyboardMenu(page, "Workspace Northstar. Switch workspace", "Orbit");
    await page
      .getByRole("button", {
        name: "Workspace Northstar. Switch workspace",
        exact: true,
      })
      .click();
    // Firna exposes footer actions with different roles across viewports.
    const add = page.getByText("Add an organization", { exact: true });
    if (item.name === "admin") await visible(add);
    assert.equal(await add.count(), item.name === "admin" ? 1 : 0);
    await page.keyboard.press("Escape");
    if (item.name.endsWith("-claim")) {
      await page
        .getByRole("button", { name: "Account of mira", exact: true })
        .click();
      await page
        .getByRole("menuitem", { name: "Claim the Admin role", exact: true })
        .click();
      await visible(page.getByRole("link", { name: "Settings", exact: true }));
    }
    assert.equal(
      await page.getByRole("link", { name: "Settings", exact: true }).count(),
      item.name === "admin" || item.name.endsWith("-claim") ? 1 : 0,
    );
  }
  if (item.name === "no-role-claim") {
    await page
      .getByRole("button", { name: "Claim the Admin role", exact: true })
      .click();
    await visible(
      page.getByRole("heading", { name: "Set up ciproxy", exact: true }),
    );
  }
  if (item.name === "no-role" || item.name === "access-ended") {
    assert.equal(
      await page.getByRole("navigation", { name: "Main", exact: true }).count(),
      0,
    );
  }
  if (item.name === "app-removed") {
    await visible(page.getByText(/Install it again before 6 December/));
    assert.equal(
      await page
        .getByRole("link", { name: "Install on GitHub", exact: true })
        .getAttribute("href"),
      "/__fixture/github/install",
    );
  }
  if (item.name === "personal") {
    assert.equal(
      await page
        .getByRole("link", { name: "Remove from mira", exact: true })
        .getAttribute("href"),
      "https://github.com/settings/installations/9002",
    );
  }
  if (item.name === "sign-in") {
    const signIn = page.getByRole("link", {
      name: "Sign in with GitHub",
      exact: true,
    });
    assert.equal(await signIn.getAttribute("href"), "/api/auth/github");
    await signIn.click();
    await visible(
      page.getByRole("heading", {
        name: "Fixture navigation complete",
        exact: true,
      }),
    );
    assert.ok(
      (await facts(origin)).some(
        (fact) =>
          fact.path === "/api/auth/github" && fact.kind === "navigation",
      ),
    );
  }
}

export async function copyFailure(page: Page, origin: string): Promise<void> {
  await control(origin, { scenario: "setup-waiting" });
  await page.goto(`${origin}/setup?workspace=${workspaceId}`, {
    waitUntil: "domcontentloaded",
  });
  await page
    .getByRole("button", { name: "Copy the install command", exact: true })
    .click();
  await visible(
    page.getByText("Unable to copy the install command", { exact: true }),
  );
}
