// The browser tests of the docs, on a fresh production build of the site:
// the search dialog, and the layout of the picture of How ciproxy works. They
// stay in this one file, so that the suite builds the site once. `node --test`
// runs each test file in its own process at the same time, and site builds
// must run one at a time.

import { deepStrictEqual, strictEqual } from "node:assert";
import { after, before, test } from "node:test";

import { chromium, type Browser, type Locator } from "playwright-core";

import { jobPath } from "../../src/data/job-path.ts";
import { placesOfBoxes, type Place } from "./job-path-places.ts";
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
 * Opens How ciproxy works in a window of this width, and gives the place of
 * each box with these titles, from the box before it.
 */
async function jobPathPlaces(
  width: number,
  titles: readonly string[],
): Promise<Place[]> {
  const context = await browser.newContext({
    viewport: { width, height: 900 },
    locale: "en-GB",
    timezoneId: "UTC",
  });
  try {
    const page = await context.newPage();
    await page.goto(`${site.origin}/docs/how-ciproxy-works/`);
    return await placesOfBoxes(page, titles);
  } finally {
    await context.close();
  }
}

/** The titles of the boxes 01 to 04, in the order of their steps. */
const stepTitles = jobPath.steps.map((box) => box.title);

test("How ciproxy works shows the first three steps in a row in a wide article", async () => {
  // Under 800 px the docs sidebar is behind the menu button, so the picture
  // is 672 px wide in a window of 720 px.
  const titles = stepTitles.slice(0, 3);
  deepStrictEqual(
    await jobPathPlaces(720, titles),
    ["right", "right"],
    `${titles.join(", ")} must form a row from left to right`,
  );
});

test("How ciproxy works stacks the job path in one column beside the docs sidebar", async () => {
  // From 800 px the docs sidebar shows, so the picture is 456 px wide in a
  // window of 840 px.
  const titles = [...stepTitles, jobPath.boat.title];
  deepStrictEqual(
    await jobPathPlaces(840, titles),
    ["below", "below", "below", "below"],
    `${titles.join(", ")} must stack in one column`,
  );
});
