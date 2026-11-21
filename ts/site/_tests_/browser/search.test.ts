import { strictEqual } from "node:assert";
import { after, before, test } from "node:test";

import { chromium, type Browser, type Locator } from "playwright-core";

import { serveSite } from "./site.ts";

let site: Awaited<ReturnType<typeof serveSite>>;
let browser: Browser;

before(async () => {
  site = await serveSite();
  browser = await chromium.launch({ channel: "chrome" });
});

after(async () => {
  await browser?.close();
  await site?.close();
});

async function assertFocused(locator: Locator, message: string): Promise<void> {
  await locator.waitFor();
  strictEqual(
    await locator.evaluate(
      (element) => element === element.ownerDocument.activeElement,
    ),
    true,
    message,
  );
}

test("same-page search continues keyboard focus at the opened heading", async () => {
  const context = await browser.newContext({
    viewport: { width: 1440, height: 900 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  try {
    const page = await context.newPage();
    await page.goto(`${site.origin}/docs/`);
    await page
      .getByRole("button", { name: "Search docs", exact: true })
      .click();
    const input = page.getByRole("combobox", { name: "Search docs" });
    await input.fill("runs-on");
    await page.getByRole("grid", { name: "Search results" }).waitFor();
    await input.press("ArrowUp");
    await input.press("Enter");
    await page.waitForURL(`${site.origin}/docs/#how-a-job-runs`);
    await page
      .getByRole("dialog", { name: "Search docs" })
      .waitFor({ state: "hidden" });
    await page.keyboard.press("Tab");
    await assertFocused(
      page.getByRole("link", {
        name: "Section titled “How a job runs”",
        exact: true,
      }),
      "Tab must continue at the opened heading link",
    );
  } finally {
    await context.close();
  }
});

test("Escape and Cancel restore focus to the search button", async () => {
  const context = await browser.newContext({
    viewport: { width: 390, height: 900 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  try {
    const page = await context.newPage();
    await page.goto(`${site.origin}/docs/`);
    const open = page.getByRole("button", { name: "Search docs", exact: true });
    const dialog = page.getByRole("dialog", { name: "Search docs" });
    await open.click();
    await page.getByRole("combobox", { name: "Search docs" }).press("Escape");
    await dialog.waitFor({ state: "hidden" });
    await assertFocused(open, "Escape must restore focus to the search button");
    await open.click();
    await page.getByRole("button", { name: "Cancel", exact: true }).click();
    await dialog.waitFor({ state: "hidden" });
    await assertFocused(open, "Cancel must restore focus to the search button");
  } finally {
    await context.close();
  }
});
