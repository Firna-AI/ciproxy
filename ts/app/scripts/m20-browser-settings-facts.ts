import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";

/** A phone shows cards; a wider window shows tables with column headers. */
function phone(width: number): boolean {
  return width < 640;
}

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
