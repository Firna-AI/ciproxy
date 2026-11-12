import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";

export async function shellJourney(
  page: Page,
  origin: string,
  journey: "add" | "switch" | "sign-out",
): Promise<void> {
  await control(origin, { scenario: journey === "add" ? "admin" : "viewer" });
  await page.goto(`${origin}/jobs?workspace=${workspaceId}`, {
    waitUntil: "domcontentloaded",
  });
  await visible(page.getByRole("heading", { name: "Jobs", exact: true }));
  if (journey === "sign-out") {
    await page
      .getByRole("button", { name: "Account of mira", exact: true })
      .click();
    await page.getByRole("menuitem", { name: "Sign out", exact: true }).click();
    await visible(
      page.getByRole("link", { name: "Sign in with GitHub", exact: true }),
    );
    assert.equal(
      await page.getByRole("navigation", { name: "Main", exact: true }).count(),
      0,
    );
    assert.ok(
      (await facts(origin)).some(
        (fact) => fact.path === "/api/auth/sign-out" && fact.status === 200,
      ),
    );
    return;
  }
  const trigger = page.getByRole("button", {
    name: "Workspace Northstar. Switch workspace",
    exact: true,
  });
  await trigger.focus();
  await trigger.press("Enter");
  await visible(page.getByRole("menu", { name: "Workspaces", exact: true }));
  if (journey === "switch") {
    // The menu opens on the current workspace. ArrowDown selects Orbit.
    await page.keyboard.press("ArrowDown");
    await page.keyboard.press("Enter");
    await visible(
      page.getByRole("heading", {
        name: "Waiting for an Admin of Orbit",
        exact: true,
      }),
    );
  } else {
    // Firna exposes footer actions with different roles across viewports.
    const add = page.getByText("Add an organization", { exact: true });
    await visible(add);
    await add.click();
    await visible(
      page.getByRole("heading", {
        name: "Fixture navigation complete",
        exact: true,
      }),
    );
    assert.ok(
      (await facts(origin)).some(
        (fact) =>
          fact.path.endsWith("/organizations/add") &&
          fact.method === "POST" &&
          fact.status === 200,
      ),
    );
    assert.equal(new URL(page.url()).pathname, "/__fixture/github/install");
  }
}

export async function staticReloads(page: Page, origin: string): Promise<void> {
  await control(origin, { scenario: "admin" });
  const titles = {
    jobs: "Jobs",
    machines: "Machines",
    savings: "Savings",
    billing: "Plan and usage",
    settings: "Settings",
  } as const;
  for (const [path, title] of Object.entries(titles)) {
    await page.goto(`${origin}/${path}?workspace=${workspaceId}`, {
      waitUntil: "domcontentloaded",
    });
    await page.reload({ waitUntil: "domcontentloaded" });
    await visible(page.getByRole("heading", { name: title, exact: true }));
    const loaded =
      path === "jobs"
        ? "2 warm runners are ready"
        : path === "savings"
          ? "Your cost with ciproxy"
          : path === "billing"
            ? "This period"
            : "This page is coming soon";
    await visible(page.getByRole("heading", { name: loaded, exact: true }));
  }
}
