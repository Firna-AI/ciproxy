import assert from "node:assert/strict";
import { resolve } from "node:path";

import type { BrowserResponse, Locator, Page } from "./browser-driver.ts";
import { insideViewport, visible } from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fields } from "./harness-state.ts";
import { pageRoutes, type PageRoute } from "./harness-pages-state.ts";

const successEvidence = new WeakMap<Page, boolean>();

export function configureSuccessEvidence(page: Page, enabled: boolean): void {
  successEvidence.set(page, enabled);
}

export async function pageViewport(page: Page, width: number): Promise<void> {
  for (const role of [
    "heading",
    "button",
    "link",
    "radio",
    "tab",
    "combobox",
    "textbox",
    "spinbutton",
    "dialog",
    "alert",
    "main",
  ]) {
    for (const locator of await page.getByRole(role).all()) {
      await insideViewport(locator, width);
    }
  }
}

export async function capturePage(
  page: Page,
  output: string,
  name: string,
  width: number,
): Promise<string | undefined> {
  await pageViewport(page, width);
  if (successEvidence.get(page) === false) return undefined;
  const path = resolve(output, `${name}-${width}.png`);
  await page.screenshot({ path, fullPage: true });
  return path;
}

export async function absent(locator: Locator): Promise<void> {
  await locator.waitFor({ state: "hidden" });
  assert.equal(await locator.count(), 0, "The action must be absent.");
}

export async function fieldValue(
  locator: Locator,
  expected: string,
): Promise<void> {
  await visible(locator);
  assert.equal(await locator.inputValue(), expected);
}

export async function apiAction(
  page: Page,
  path: string,
  method: string,
  action: () => Promise<void>,
): Promise<BrowserResponse> {
  const pending = page.waitForResponse(
    (response) =>
      new URL(response.url()).pathname ===
        `/api/workspaces/${workspaceId}/${path}` &&
      response.request().method() === method,
  );
  const [response] = await Promise.all([pending, action()]);
  await response.finished();
  return response;
}

export async function heldPages(origin: string): Promise<PageRoute[]> {
  const response = await fetch(`${origin}/__fixture/state`);
  assert.equal(response.status, 200, "The fixture state read failed.");
  const value: unknown = await response.json();
  if (
    !fields(value) ||
    !fields(value.pages) ||
    !Array.isArray(value.pages.pending)
  ) {
    throw new Error("The fixture pending-request state is invalid.");
  }
  return value.pages.pending.map((item: unknown) => {
    const route = pageRoutes.find((candidate) => candidate === item);
    if (route === undefined)
      throw new Error("The fixture pending route is invalid.");
    return route;
  });
}

export async function visibleBox(locator: Locator) {
  await visible(locator);
  const box = await locator.boundingBox();
  assert.ok(
    box !== null && box.width > 0 && box.height > 0,
    "The rendered item must have visible dimensions.",
  );
  return box;
}

export async function insideRegion(
  region: Locator,
  item: Locator,
): Promise<void> {
  const outer = await visibleBox(region);
  const inner = await visibleBox(item);
  assert.ok(
    inner.x >= outer.x - 1 &&
      inner.y >= outer.y - 1 &&
      inner.x + inner.width <= outer.x + outer.width + 1 &&
      inner.y + inner.height <= outer.y + outer.height + 1,
    "The row must stay inside its named panel.",
  );
}

export async function separateRegions(
  first: Locator,
  second: Locator,
): Promise<void> {
  const a = await visibleBox(first);
  const b = await visibleBox(second);
  assert.ok(
    a.x + a.width <= b.x + 1 ||
      b.x + b.width <= a.x + 1 ||
      a.y + a.height <= b.y + 1 ||
      b.y + b.height <= a.y + 1,
    "Named panels must not overlap.",
  );
}
