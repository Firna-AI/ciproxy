// The journeys of Settings under Overflow: the switch, the wait, and the
// limit save at once and stay after a reload; a connection, a reconnect, a
// change of key, and a disconnect move focus to the Boat panel; a change of
// key keeps a limit that the limit panel saved while it was open; failed
// changes keep the typed values; an accepted connection is never sent again.
import assert from "node:assert/strict";

import {
  connections,
  fieldFocused,
  fillConnection,
  focusedOn,
  overflowPath,
} from "./browser-boat.ts";
import type { Page } from "./browser-driver.ts";
import { control, viewportControls, visible } from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import type { ScenarioName } from "./harness-scenarios.ts";
import { fixtureBoatKey } from "./fixtures-boat.ts";

const limitLabel = "Monthly spend limit, US dollars";

/** Another width with the same frame: phone, or desktop with its sidebar. */
function sameFrameWidth(width: number): number {
  if (width < 640) return 600;
  return width === 720 ? 1440 : 720;
}
const connected = "Connected to the account northstar";

async function open(page: Page, origin: string, scenario: ScenarioName) {
  await control(origin, { scenario });
  await page.goto(origin + overflowPath, { waitUntil: "domcontentloaded" });
}

async function policy(page: Page, origin: string, width: number) {
  await open(page, origin, "overflow-connected");
  await visible(page.getByText(connected, { exact: true }));
  await viewportControls(page, width);
  const toggle = (checked: boolean) =>
    page.getByRole("switch", {
      name: "Use Boat when your machines are busy",
      checked,
    });
  await toggle(true).click();
  await visible(toggle(false));
  await page
    .getByRole("button", {
      name: "Start on Boat after a job waits, 30 seconds",
    })
    .click();
  await page.getByRole("option", { name: "1 minute", exact: true }).click();
  await visible(
    page.getByRole("button", {
      name: "Start on Boat after a job waits, 1 minute",
    }),
  );
  await page.getByLabel("Limit, US dollars", { exact: true }).fill("250.50");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await visible(page.getByText("Limit $250.50", { exact: true }));
  await focusedOn(page, "Save");
  await page.reload({ waitUntil: "domcontentloaded" });
  await visible(page.getByText("Limit $250.50", { exact: true }));
  await visible(toggle(false));
  await visible(
    page.getByRole("button", {
      name: "Start on Boat after a job waits, 1 minute",
    }),
  );
}

async function connect(page: Page, origin: string, width: number) {
  await open(page, origin, "overflow-not-connected");
  await visible(page.getByLabel("Boat API key", { exact: true }));
  await viewportControls(page, width);
  await fillConnection(page, "Boat API key", fixtureBoatKey, "150");
  await visible(page.getByText(connected, { exact: true }));
  await focusedOn(page, "Boat");
  await visible(page.getByText("Limit $150", { exact: true }));
  assert.deepEqual(await connections(origin), [200]);
}

async function reconnect(page: Page, origin: string) {
  await open(page, origin, "overflow-rejected");
  const limit = page.getByLabel(limitLabel, { exact: true });
  await visible(limit);
  assert.equal(await limit.inputValue(), "200", "The saved limit stays");
  await visible(
    page.getByText(
      "Your saved limit. At the limit, jobs run on your machines only until the next month.",
      { exact: true },
    ),
  );
  await fillConnection(page, "New Boat API key", fixtureBoatKey, null);
  await visible(page.getByText(connected, { exact: true }));
  assert.equal(
    await page.getByText("Boat rejected the API key on 12 November").count(),
    0,
  );
}

async function changeKey(page: Page, origin: string) {
  await open(page, origin, "overflow-connected");
  const change = page.getByRole("button", { name: "Change key", exact: true });
  await change.click();
  await fieldFocused(page, "New Boat API key");
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await focusedOn(page, "Boat");
  await change.click();
  // The limit panel saves a lower limit while the key change stays open.
  await page.getByLabel("Limit, US dollars", { exact: true }).fill("100");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await visible(page.getByText("Limit $100", { exact: true }));
  await fillConnection(page, "New Boat API key", fixtureBoatKey, null);
  await visible(page.getByText(connected, { exact: true }));
  await focusedOn(page, "Boat");
  assert.deepEqual(await connections(origin), [200]);
  // The key change left its limit untouched, so the newer saved limit stays.
  assert.equal(await page.getByText(/^Limit \$/).textContent(), "Limit $100");
}

async function disconnect(page: Page, origin: string, width: number) {
  await open(page, origin, "overflow-connected");
  const button = page.getByRole("button", { name: "Disconnect", exact: true });
  await button.click();
  const dialog = page.getByRole("dialog", { name: "Disconnect Boat?" });
  await visible(dialog);
  await visible(
    page.getByText(
      "Jobs that run on Boat now finish. ciproxy then deletes the key.",
      { exact: true },
    ),
  );
  await focusedOn(page, "Cancel");
  await page.keyboard.press("Escape");
  await dialog.waitFor({ state: "hidden" });
  await focusedOn(page, "Disconnect");
  await button.click();
  await visible(dialog);
  // A resize that keeps the frame of the page keeps the dialog and its focus.
  await page.setViewportSize({ width: sameFrameWidth(width), height: 1000 });
  await focusedOn(page, "Cancel");
  await page.setViewportSize({ width, height: 1000 });
  await focusedOn(page, "Cancel");
  await button.click();
  await dialog.waitFor({ state: "hidden" });
  const limit = page.getByLabel(limitLabel, { exact: true });
  await visible(limit);
  assert.equal(await limit.inputValue(), "200");
  await focusedOn(page, "Boat");
}

async function saveFailure(page: Page, origin: string) {
  await open(page, origin, "overflow-save-failure");
  const limit = page.getByLabel("Limit, US dollars", { exact: true });
  await limit.fill("300");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await visible(page.getByText("Unable to save the limit", { exact: true }));
  assert.equal(await limit.inputValue(), "300");
}

async function connectFailure(page: Page, origin: string) {
  await open(page, origin, "overflow-connect-failure");
  await fillConnection(page, "Boat API key", fixtureBoatKey, "200");
  await visible(page.getByText("Unable to connect Boat", { exact: true }));
  assert.equal(
    await page.getByLabel("Boat API key", { exact: true }).inputValue(),
    fixtureBoatKey,
    "A connection that did not finish keeps the typed key",
  );
}

async function refreshAfterConnect(page: Page, origin: string) {
  await open(page, origin, "overflow-refresh-after-connect");
  await fillConnection(page, "Boat API key", fixtureBoatKey, "200");
  await visible(
    page.getByText("Boat connected. Check the connection, then try again.", {
      exact: true,
    }),
  );
  assert.equal(await page.getByLabel("Boat API key").count(), 0);
  await page.getByRole("button", { name: "Try again", exact: true }).click();
  await visible(page.getByText(connected, { exact: true }));
  assert.deepEqual(await connections(origin), [200]);
}

export function createOverflowRuns(origin: string): readonly SmokeRun[] {
  const run = (
    name: string,
    check: (page: Page, origin: string, width: number) => Promise<void>,
  ): SmokeRun => ({
    name,
    check: (page, _context, width) => check(page, origin, width),
  });
  return [
    run("overflow-policy", policy),
    run("overflow-connect", connect),
    run("overflow-reconnect", reconnect),
    run("overflow-change-key", changeKey),
    run("overflow-disconnect", disconnect),
    run("overflow-save-failure", saveFailure),
    run("overflow-connect-failure", connectFailure),
    run("overflow-refresh-after-connect", refreshAfterConnect),
  ];
}
