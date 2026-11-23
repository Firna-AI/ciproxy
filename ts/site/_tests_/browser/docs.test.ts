// The browser tests of the docs, on a fresh production build of the site:
// the search dialog, and the layout of the picture of How ciproxy works. They
// stay in this one file, so that the suite builds the site once. `node --test`
// runs each test file in its own process at the same time, and site builds
// must run one at a time.

import { deepStrictEqual, strictEqual } from "node:assert";
import { after, before, test } from "node:test";

import { chromium, type Browser, type Locator } from "playwright-core";

import { tableOfBoxes, type Cell } from "./job-path-places.ts";
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

/**
 * Opens How ciproxy works in a window of this width, and gives the table of
 * the boxes of its picture: the part of each box in its row and column.
 */
async function jobPathTable(width: number): Promise<Cell[][]> {
  const context = await browser.newContext({
    viewport: { width, height: 900 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  try {
    const page = await context.newPage();
    await page.goto(`${site.origin}/docs/how-ciproxy-works/`);
    return await tableOfBoxes(page);
  } finally {
    await context.close();
  }
}

test("How ciproxy works shows the path in a row, with Boat under ciproxy and the machine under the runner, in a wide article", async () => {
  // Under 800 px the docs sidebar is behind the menu button, so the picture
  // is 672 px wide in a window of 720 px.
  deepStrictEqual<Cell[][]>(await jobPathTable(720), [
    ["github", "ciproxy", "runner"],
    ["empty", "boat", "machine"],
  ]);
});

test("How ciproxy works stacks the five boxes in one column beside the docs sidebar", async () => {
  // From 800 px the docs sidebar shows, so the picture is 456 px wide in a
  // window of 840 px.
  deepStrictEqual<Cell[][]>(await jobPathTable(840), [
    ["github"],
    ["ciproxy"],
    ["runner"],
    ["machine"],
    ["boat"],
  ]);
});
