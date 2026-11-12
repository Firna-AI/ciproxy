import assert from "node:assert/strict";

import type { Locator, Page } from "./browser-driver.ts";
import { fields, type RequestFact } from "./harness-state.ts";
import type { ScenarioName } from "./harness-scenarios.ts";
import type { PagesControl } from "./harness-pages-controls.ts";

export type HarnessControl = {
  scenario?: ScenarioName;
  build?: string;
  reset?: boolean;
  /** Answer a held Boat connection. */
  release?: boolean;
  pages?: PagesControl;
};

export async function control(
  origin: string,
  value: HarnessControl,
): Promise<void> {
  const result = await fetch(`${origin}/__fixture/control`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(value),
  });
  assert.equal(result.status, 200, "Fixture control failed");
}

function isFact(value: unknown): value is RequestFact {
  return (
    fields(value) &&
    typeof value.method === "string" &&
    typeof value.path === "string" &&
    typeof value.status === "number" &&
    ["api", "document", "navigation"].includes(String(value.kind))
  );
}

export async function facts(origin: string): Promise<RequestFact[]> {
  const result = await fetch(`${origin}/__fixture/state`);
  const value: unknown = await result.json();
  if (
    !fields(value) ||
    !Array.isArray(value.facts) ||
    !value.facts.every(isFact)
  )
    throw new Error("Invalid harness facts");
  return value.facts;
}

export async function visible(locator: Locator): Promise<void> {
  await locator.waitFor({ state: "visible" });
}

export async function insideViewport(
  locator: Locator,
  width: number,
): Promise<void> {
  await visible(locator);
  const box = await locator.boundingBox();
  assert.notEqual(box, null, "The screen has no visible bounds");
  assert.ok(
    box !== null && box.x >= -1 && box.x + box.width <= width + 1,
    "The screen exceeds the viewport",
  );
}

export async function viewportControls(
  page: Page,
  width: number,
): Promise<void> {
  for (const role of ["heading", "button", "link", "radio"]) {
    for (const locator of await page.getByRole(role).all())
      await insideViewport(locator, width);
  }
}

export async function documentReloaded(
  origin: string,
  before: number,
  destination: string,
): Promise<void> {
  const after = await facts(origin);
  const document = after.findIndex(
    (fact, index) =>
      index >= before && fact.kind === "document" && fact.path === destination,
  );
  assert.ok(
    document >= before,
    `No full-document navigation to ${destination}`,
  );
  const firstApi = after.findIndex(
    (fact, index) => index >= before && fact.kind === "api",
  );
  assert.ok(
    firstApi === -1 || firstApi > document,
    "The destination called the API before its document reloaded",
  );
}

export async function keyboardMenu(
  page: Page,
  trigger: string,
  item: string,
): Promise<void> {
  const button = page.getByRole("button", { name: trigger, exact: true });
  await button.focus();
  await button.press("Enter");
  await visible(page.getByRole("menu"));
  await visible(page.getByRole("menuitem", { name: item, exact: true }));
  await page.keyboard.press("ArrowDown");
  await page.keyboard.press("Escape");
}
