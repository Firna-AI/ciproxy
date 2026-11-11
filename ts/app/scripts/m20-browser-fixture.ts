import assert from "node:assert/strict";
import { resolve } from "node:path";

import type { BrowserContext, Locator, Page } from "./browser-driver.ts";
import { roleNames } from "../src/view/format.ts";
import {
  insideViewport,
  visible,
  viewportControls,
} from "./browser-fixture.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fields } from "./harness-state.ts";
import type { M20Scenario } from "./m20-harness-state.ts";

export type M20Case = {
  name: string;
  expectedHttp?: readonly number[];
  check(page: Page, context: BrowserContext, width: number): Promise<void>;
};

type InputLocator = Locator & { fill(value: string): Promise<void> };
type ScopedLocator = Locator & Pick<Page, "getByRole" | "getByText">;

function editable(value: unknown): value is InputLocator {
  return fields(value) && typeof value.fill === "function";
}

function scoped(value: unknown): value is ScopedLocator {
  return (
    fields(value) &&
    typeof value.getByRole === "function" &&
    typeof value.getByText === "function"
  );
}

export function within(locator: Locator): ScopedLocator {
  if (!scoped(locator))
    throw new Error("The browser locator cannot scope a control");
  return locator;
}

export async function fill(
  page: Page,
  name: string,
  value: string,
): Promise<void> {
  const locator = page.getByRole("textbox", { name, exact: true });
  await visible(locator);
  if (!editable(locator))
    throw new Error("The browser locator cannot fill an input");
  await locator.fill(value);
}

export async function selectRole(
  page: Page,
  login: string,
  role: "admin" | "billing" | "viewer",
): Promise<void> {
  const locator = page.getByRole("button", {
    name: `Role of ${login}`,
    exact: true,
  });
  await visible(locator);
  await locator.click();
  const option = page.getByRole("option", {
    name: roleNames[role],
    exact: true,
  });
  await visible(option);
  await option.click();
}

export async function m20Control(
  origin: string,
  scenario: M20Scenario,
  reset = true,
): Promise<void> {
  const result = await fetch(`${origin}/__fixture/control`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ scenario, reset }),
  });
  assert.equal(result.status, 200, "Fixture control failed");
}

export function machinesUrl(
  origin: string,
  machine?: string,
  add = false,
): string {
  const query = new URLSearchParams({ workspace: workspaceId });
  if (machine) query.set("machine", machine);
  if (add) query.set("view", "add");
  return `${origin}/machines?${query}`;
}

export function settingsUrl(
  origin: string,
  tab?: string,
  organization?: string,
): string {
  const query = new URLSearchParams({ workspace: workspaceId });
  if (tab) query.set("tab", tab);
  if (organization) query.set("organization", organization);
  return `${origin}/settings?${query}`;
}

export async function open(
  page: Page,
  origin: string,
  scenario: M20Scenario,
  url: string,
  heading: string,
): Promise<void> {
  await m20Control(origin, scenario);
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await visible(page.getByRole("heading", { name: heading, exact: true }));
}

export async function capture(
  page: Page,
  width: number,
  output: string,
  name: string,
): Promise<void> {
  await viewportControls(page, width);
  for (const role of ["textbox", "checkbox", "switch", "combobox"])
    for (const locator of await page.getByRole(role).all())
      await insideViewport(locator, width);
  await page.screenshot({
    path: resolve(output, `${name}-${width}.png`),
    fullPage: true,
  });
}

export async function dialogCancel(page: Page, title: string): Promise<void> {
  const dialog = page.getByRole("dialog", { name: title, exact: true });
  await visible(dialog);
  await within(dialog)
    .getByRole("button", { name: "Cancel", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden" });
}
