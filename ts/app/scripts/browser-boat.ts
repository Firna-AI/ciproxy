// The Boat journeys of the browser smoke: real forms and changes against
// the fixture API, the focus after each change, and the navigation between
// Setup, Settings, and Settings under Overflow. The Setup connection also
// checks that the typed key stays masked and private. The rules behind the
// copy and the state are Node tests.
import assert from "node:assert/strict";

import { overflowHref, workspaceHref } from "../src/routing/pages.ts";
import { keyRefused } from "../src/view/boat-failures.ts";
import { settingsHref } from "../src/view/settings-tabs.ts";
import {
  assertKeyMasked,
  assertKeyPrivate,
  watchKeyLines,
} from "./browser-boat-privacy.ts";
import type { Page } from "./browser-driver.ts";
import {
  control,
  facts,
  viewportControls,
  visible,
} from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import { fixtureBoatKey, refusedBoatKey } from "./fixtures-boat.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fields } from "./harness-state.ts";

export const setupPath = workspaceHref("setup", workspaceId);
export const overflowPath = overflowHref(workspaceId);

/** The HTTP failure that each Boat journey causes on purpose. */
export const boatExpectedStatus: ReadonlyMap<string, number> = new Map([
  ["boat-setup-connect", 422],
  ["overflow-save-failure", 503],
  ["overflow-connect-failure", 503],
  ["overflow-refresh-after-connect", 503],
]);

/** Wait until focus rests on the element with exactly this text. */
export async function focusedOn(page: Page, name: string): Promise<void> {
  await page.waitForFunction(
    (text) => document.activeElement?.textContent?.trim() === text,
    name,
  );
}

/** Wait until focus rests in the field with this visible label. */
export async function fieldFocused(page: Page, label: string): Promise<void> {
  await page.waitForFunction((text) => {
    const id = document.activeElement?.getAttribute("aria-labelledby");
    return id ? document.getElementById(id)?.textContent === text : false;
  }, label);
}

/** Type the key, and the limit unless it is null. */
async function typeKey(
  page: Page,
  keyLabel: string,
  key: string,
  limit: string | null,
): Promise<void> {
  await page.getByLabel(keyLabel, { exact: true }).fill(key);
  if (limit !== null) {
    await page
      .getByLabel("Monthly spend limit, US dollars", { exact: true })
      .fill(limit);
  }
}

/** Type the key and the limit, then press Check key. */
export async function checkKey(
  page: Page,
  keyLabel: string,
  key: string,
  limit: string | null,
): Promise<void> {
  await typeKey(page, keyLabel, key, limit);
  await page.getByRole("button", { name: "Check key", exact: true }).click();
}

/** Check the key, keep the likely wallet, and press Connect Boat. */
export async function fillConnection(
  page: Page,
  keyLabel: string,
  key: string,
  limit: string | null,
): Promise<void> {
  await checkKey(page, keyLabel, key, limit);
  await visible(
    page.getByRole("radiogroup", { name: "Wallet that Boat bills" }),
  );
  await page.getByRole("button", { name: "Connect Boat", exact: true }).click();
}

/** The status of each connection request, in order. */
export async function connections(origin: string): Promise<number[]> {
  return (await facts(origin))
    .filter(
      (fact) =>
        fact.method === "POST" && fact.path.endsWith("/overflow/connection"),
    )
    .map((fact) => fact.status);
}

/** The status of each key check, in order. */
export async function checks(origin: string): Promise<number[]> {
  return (await facts(origin))
    .filter(
      (fact) =>
        fact.method === "POST" && fact.path.endsWith("/overflow/wallets"),
    )
    .map((fact) => fact.status);
}

/** A record of the harness: field names or the wallet, never the key. */
export async function harnessRecord(
  origin: string,
  name: "boatConnectionFields" | "boatConnectionWallet",
): Promise<unknown> {
  const value: unknown = await (
    await fetch(`${origin}/__fixture/state`)
  ).json();
  return fields(value) ? value[name] : undefined;
}

async function setupConnect(page: Page, origin: string, width: number) {
  const keyLines = watchKeyLines(page);
  const keyField = "Boat API key";
  await control(origin, { scenario: "boat-setup" });
  await page.goto(origin + setupPath, { waitUntil: "domcontentloaded" });
  await visible(page.getByLabel(keyField, { exact: true }));
  await viewportControls(page, width);
  await typeKey(page, keyField, refusedBoatKey, "200");
  await assertKeyMasked(page, keyField, "before the key check");
  await assertKeyPrivate(page, keyLines, "before the key check");
  await page.getByRole("button", { name: "Check key", exact: true }).click();
  await visible(page.getByText(keyRefused, { exact: true }));
  assert.equal(
    await page
      .getByLabel("Monthly spend limit, US dollars", { exact: true })
      .inputValue(),
    "200",
    "A refused key keeps the typed limit",
  );
  await assertKeyMasked(page, keyField, "after the refused key");
  await assertKeyPrivate(page, keyLines, "after the refused key");
  await checkKey(page, keyField, fixtureBoatKey, null);
  await visible(
    page.getByRole("radiogroup", { name: "Wallet that Boat bills" }),
  );
  await assertKeyMasked(page, keyField, "before Connect Boat");
  await assertKeyPrivate(page, keyLines, "before Connect Boat");
  await page.getByRole("button", { name: "Connect Boat", exact: true }).click();
  await visible(
    page.getByText(
      "Connected to the Boat account northstar · spend limit $200 a month",
      { exact: true },
    ),
  );
  await focusedOn(page, "Run the first job");
  assert.equal(await page.getByLabel("Boat API key").count(), 0);
  await assertKeyPrivate(page, keyLines, "after the connection");
  assert.deepEqual(await checks(origin), [422, 200]);
  assert.deepEqual(await connections(origin), [200]);
  assert.deepEqual(await harnessRecord(origin, "boatConnectionFields"), [
    "api_key",
    "spend_limit_cents",
    "wallet",
  ]);
  assert.equal(
    await harnessRecord(origin, "boatConnectionWallet"),
    "boat-org-1001",
    "Setup keeps the likely wallet",
  );
}

async function setupSkip(page: Page, origin: string) {
  await control(origin, { scenario: "boat-setup" });
  await page.goto(origin + setupPath, { waitUntil: "domcontentloaded" });
  await page.getByRole("button", { name: "Skip for now", exact: true }).click();
  const link = page.getByRole("link", {
    name: "Settings under Overflow",
    exact: true,
  });
  await visible(link);
  await focusedOn(page, "Run the first job");
  await visible(
    page.getByText("Waiting for a job with runs-on: ciproxy…", { exact: true }),
  );
  await page.reload({ waitUntil: "domcontentloaded" });
  await visible(link);
  assert.equal(await page.getByLabel("Boat API key").count(), 0);
  await link.click();
  await page.waitForURL(/\/settings\/overflow\?workspace=/);
  await visible(page.getByLabel("Boat API key", { exact: true }));
  assert.deepEqual(await connections(origin), []);
}

async function settingsTabs(page: Page, origin: string, width: number) {
  await control(origin, { scenario: "admin" });
  await page.goto(origin + workspaceHref("settings", workspaceId), {
    waitUntil: "domcontentloaded",
  });
  const labels = page.getByRole("button", { name: "Add label", exact: true });
  await visible(labels);
  await viewportControls(page, width);
  await page.getByRole("link", { name: "Overflow", exact: true }).click();
  await page.waitForURL(/\/settings\/overflow\?workspace=/);
  await visible(page.getByLabel("Boat API key", { exact: true }));
  assert.equal(
    await page
      .getByRole("link", { name: "Overflow", exact: true })
      .getAttribute("aria-current"),
    "page",
  );
  for (const [name, tab] of [
    ["Organizations", "organizations"],
    ["Members", "members"],
  ] as const) {
    assert.equal(
      await page.getByRole("link", { name, exact: true }).getAttribute("href"),
      settingsHref(workspaceId, tab),
      `The ${name} tab of Settings under Overflow opens its Settings tab`,
    );
  }
  await page.getByRole("link", { name: "Labels", exact: true }).click();
  await visible(labels);
  assert.equal(new URL(page.url()).pathname, "/settings");
}

export function createBoatRuns(origin: string): readonly SmokeRun[] {
  return [
    {
      name: "boat-setup-connect",
      check: (page, _context, width) => setupConnect(page, origin, width),
    },
    { name: "boat-setup-skip", check: (page) => setupSkip(page, origin) },
    {
      name: "settings-tabs",
      check: (page, _context, width) => settingsTabs(page, origin, width),
    },
  ];
}
