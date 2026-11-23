// The order of places in the label dialog. Remove takes a place out, Add a
// place puts it back at the end, and the keyboard moves a place with its
// grab handle. Save label sends the order, and the list then names the places
// in that order. A macOS label holds own machines only, with no handle, no
// Remove, and no Add. The words of each row are Node tests.
import assert from "node:assert/strict";

import type { Locator, Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import {
  capture,
  open,
  settingsUrl,
  within,
  type M20Case,
} from "./m20-browser-fixture.ts";

const title = "Edit ciproxy-large";

async function editLarge(page: Page, origin: string): Promise<Locator> {
  await open(page, origin, "settings", settingsUrl(origin), "Settings");
  await page.getByRole("button", { name: title, exact: true }).click();
  const dialog = page.getByRole("dialog", { name: title, exact: true });
  await visible(dialog);
  return dialog;
}

/** The names of the places of the order, in order, from their handles. */
async function order(dialog: Locator): Promise<string[]> {
  const list = within(dialog).getByRole("list", {
    name: "Where jobs of ciproxy-large run",
    exact: true,
  });
  await visible(list);
  const buttons = await within(list)
    .getByRole("button", { name: /^Remove / })
    .all();
  const labels = await Promise.all(
    buttons.map((button) => button.getAttribute("aria-label")),
  );
  return labels.map((label) => (label ?? "").replace(/^Remove /, ""));
}

async function changeOrder(
  page: Page,
  origin: string,
  width: number,
  output: string,
) {
  const dialog = await editLarge(page, origin);
  const scoped = within(dialog);
  assert.deepEqual(await order(dialog), ["Your machines", "Boat"]);
  await scoped
    .getByRole("button", { name: "Remove Boat", exact: true })
    .click();
  assert.deepEqual(await order(dialog), ["Your machines"]);
  await visible(
    scoped.getByText("Add a provider to the order to use this choice.", {
      exact: true,
    }),
  );
  await scoped.getByRole("button", { name: "Add Boat", exact: true }).click();
  assert.deepEqual(await order(dialog), ["Your machines", "Boat"]);
  const handle = scoped.getByRole("button", {
    name: "Reorder Boat",
    exact: true,
  });
  await handle.focus();
  await page.keyboard.press("Space");
  await page.keyboard.press("ArrowUp");
  await page.keyboard.press("Space");
  await page.waitForFunction((first) => {
    const rows = [...document.querySelectorAll('[aria-label^="Remove "]')];
    return rows[0]?.getAttribute("aria-label") === first;
  }, "Remove Boat");
  assert.deepEqual(await order(dialog), ["Boat", "Your machines"]);
  await capture(page, width, output, "label-order-moved");
  await scoped.getByRole("button", { name: "Save label", exact: true }).click();
  await dialog.waitFor({ state: "hidden" });
  const [placement] = await page
    .getByText("Boat, then Your machines", { exact: true })
    .all();
  assert.ok(placement, "the list names the places in the saved order");
  await visible(placement);
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: title, exact: true }).click();
  assert.deepEqual(
    await order(page.getByRole("dialog", { name: title, exact: true })),
    ["Boat", "Your machines"],
  );
}

async function emptyOrder(page: Page, origin: string) {
  const dialog = await editLarge(page, origin);
  const scoped = within(dialog);
  await scoped
    .getByRole("button", { name: "Remove Your machines", exact: true })
    .click();
  await scoped
    .getByRole("button", { name: "Remove Boat", exact: true })
    .click();
  await visible(scoped.getByText("Jobs with this label wait", { exact: true }));
  await visible(
    scoped.getByRole("button", { name: "Add Your machines", exact: true }),
  );
  await visible(scoped.getByRole("button", { name: "Add Boat", exact: true }));
}

async function macOrder(page: Page, origin: string) {
  await open(page, origin, "macs", settingsUrl(origin), "Settings");
  await page
    .getByRole("button", { name: "Edit ciproxy-macos", exact: true })
    .click();
  const dialog = page.getByRole("dialog", {
    name: "Edit ciproxy-macos",
    exact: true,
  });
  await visible(dialog);
  const scoped = within(dialog);
  await visible(
    scoped.getByText("A macOS label runs on your Macs only.", { exact: true }),
  );
  await visible(scoped.getByText("Your Macs", { exact: true }));
  assert.equal(
    await scoped.getByRole("button", { name: /^Remove / }).count(),
    0,
  );
  assert.equal(
    await scoped.getByRole("button", { name: /^Reorder / }).count(),
    0,
  );
  assert.equal(await scoped.getByRole("button", { name: /^Add / }).count(), 0);
}

export function labelOrderCases(origin: string, output: string): M20Case[] {
  return [
    {
      name: "label-order-change",
      async check(page, _context, width) {
        await changeOrder(page, origin, width, output);
        await capture(page, width, output, "label-order-saved");
      },
    },
    { name: "label-order-empty", check: (page) => emptyOrder(page, origin) },
    { name: "label-order-macos", check: (page) => macOrder(page, origin) },
  ];
}
