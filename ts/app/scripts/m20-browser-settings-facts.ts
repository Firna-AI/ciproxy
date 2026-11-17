import assert from "node:assert/strict";

import { overflowHref } from "../src/routing/pages.ts";
import { plansHref } from "../src/view/billing-routes.ts";
import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { phone } from "./m20-browser-fixture.ts";

/** The labels show where jobs run now and the jobs of 30 days. */
export async function labelFacts(page: Page, width: number): Promise<void> {
  const [placement] = await page
    .getByText("Your machines, then Boat", { exact: true })
    .all();
  assert.ok(placement, "a label runs on the machines, then Boat");
  await visible(placement);
  if (phone(width)) {
    await visible(page.getByText(/^14,371 jobs in 30\sdays$/));
    await visible(page.getByText("Job count not known yet", { exact: true }));
    return;
  }
  await visible(page.getByText("Jobs, 30 days", { exact: true }));
  await visible(page.getByText("14,371", { exact: true }));
  await visible(page.getByText("Not known yet", { exact: true }));
}

/** The organizations show the day they joined and their repository counts. */
export async function organizationFacts(page: Page): Promise<void> {
  await visible(page.getByText("Since 1 September", { exact: true }));
  await visible(page.getByText("1 of 2 private repositories", { exact: true }));
  await visible(page.getByText("All 2 private repositories", { exact: true }));
}

/** The members show names, the day of each role, and the requests. */
export async function memberFacts(page: Page, width: number): Promise<void> {
  await visible(page.getByText("Mira Chen", { exact: true }));
  if (!phone(width)) {
    const role = await page
      .getByRole("button", { name: "Role of Mira Chen", exact: true })
      .boundingBox();
    const since = await page
      .getByText("1 September", { exact: true })
      .boundingBox();
    assert.ok(role !== null && since !== null);
    assert.ok(
      since.x > role.x + role.width,
      "The grant date must have space after the role control",
    );
  }
  await visible(
    page.getByText(/^Asked on 6 November\s· member of northstar-labs$/),
  );
  await visible(
    phone(width)
      ? page.getByText("Day of the role not known", { exact: true })
      : page.getByText("Not known", { exact: true }),
  );
}

/**
 * On Free with one linked organization, the tab offers Team and says that a
 * new organization waits for it. Add stays an action, and the only linked
 * organization has no Take out.
 */
export async function freeOrganizationFacts(page: Page): Promise<void> {
  await visible(
    page.getByText("Run more organizations on Team", { exact: true }),
  );
  await visible(page.getByRole("link", { name: "Compare plans", exact: true }));
  await visible(
    page.getByText(
      /its repositories\. On Free, it stays inactive until Team\.$/,
    ),
  );
  const add = page.getByRole("button", {
    name: "Add an organization",
    exact: true,
  });
  await visible(add);
  assert.equal(await add.isEnabled(), true, "Add stays an action on Free");
  assert.equal(
    await page
      .getByRole("button", { name: "Take out northstar-labs", exact: true })
      .count(),
    0,
  );
}

/**
 * Settings under Overflow shows the tabs of Settings, and Members counts the
 * open requests of the settings read, also after a reload. Back returns to
 * the tab that opened it.
 */
export async function overflowTabFacts(
  page: Page,
  origin: string,
): Promise<void> {
  await page.getByRole("link", { name: "Overflow", exact: true }).click();
  await page.waitForURL(origin + overflowHref(workspaceId));
  const account = page.getByText("Connected to the account Northstar Boat", {
    exact: true,
  });
  const members = page.getByRole("link", {
    name: "Members, 1 open request",
    exact: true,
  });
  await visible(account);
  await visible(members);
  await page.reload({ waitUntil: "domcontentloaded" });
  await visible(account);
  await visible(members);
  await page.goBack({ waitUntil: "domcontentloaded" });
}

/** Compare plans opens Plans, and Back returns to the Free notice. */
export async function comparePlansLink(
  page: Page,
  origin: string,
): Promise<void> {
  await page.getByRole("link", { name: "Compare plans", exact: true }).click();
  await page.waitForURL(origin + plansHref(workspaceId));
  await visible(page.getByRole("heading", { name: "Plans", exact: true }));
  await visible(page.getByRole("link", { name: "Choose Team", exact: true }));
  await page.goBack({ waitUntil: "domcontentloaded" });
  await visible(
    page.getByText("Run more organizations on Team", { exact: true }),
  );
}

/** On Team, the tab offers no plan and names no hold for a new organization. */
export async function teamOrganizationFacts(page: Page): Promise<void> {
  assert.equal(
    await page
      .getByText("Run more organizations on Team", { exact: true })
      .count(),
    0,
  );
  assert.equal(await page.getByText(/stays inactive until Team/).count(), 0);
}
