// The account journeys of Settings under Overflow. A key that Boat rejected
// keeps Disconnect, with the same dialog and the saved limit for the next
// connection. While a key change runs, Cancel and every control of the page
// wait, and the accepted limit stays after the change.
import assert from "node:assert/strict";

import {
  connectionLimit,
  fillConnection,
  focusedOn,
  overflowPath,
} from "./browser-boat.ts";
import type { Page } from "./browser-driver.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import { fixtureBoatKey } from "./fixtures-boat.ts";
import type { ScenarioName } from "./harness-scenarios.ts";

async function open(page: Page, origin: string, scenario: ScenarioName) {
  await control(origin, { scenario });
  await page.goto(origin + overflowPath, { waitUntil: "domcontentloaded" });
}

/** The status of each Disconnect request, in order. */
async function disconnects(origin: string): Promise<number[]> {
  return (await facts(origin))
    .filter(
      (fact) =>
        fact.method === "DELETE" &&
        fact.path.endsWith("/overflow/providers/boat/connection"),
    )
    .map((fact) => fact.status);
}

async function rejectedDisconnect(page: Page, origin: string) {
  await open(page, origin, "overflow-rejected");
  await visible(page.getByLabel(connectionLimit, { exact: true }));
  const button = page.getByRole("button", { name: "Disconnect", exact: true });
  assert.equal(await button.count(), 1, "A rejected key keeps Disconnect");
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
  await button.click();
  await dialog.waitFor({ state: "hidden" });
  await visible(page.getByLabel("Boat API key", { exact: true }));
  assert.equal(
    await page.getByLabel(connectionLimit, { exact: true }).inputValue(),
    "200",
  );
  await focusedOn(page, "Connect an overflow provider");
  assert.deepEqual(await disconnects(origin), [200]);
}

async function changeKeyPending(page: Page, origin: string) {
  await open(page, origin, "overflow-change-key-held");
  await page.getByRole("button", { name: "Change key", exact: true }).click();
  await fillConnection(page, "New Boat API key", fixtureBoatKey, "100");
  await visible(
    page.getByRole("button", { name: "Cancel", exact: true, disabled: true }),
  );
  await visible(
    page.getByRole("switch", {
      name: "Use overflow providers",
      disabled: true,
    }),
  );
  await control(origin, { release: true });
  await visible(
    page.getByText("Connected to the account northstar", { exact: true }),
  );
  await focusedOn(page, "Boat");
  await visible(page.getByText("Limit $100", { exact: true }));
  await visible(
    page.getByRole("button", {
      name: "Save limit",
      exact: true,
      disabled: false,
    }),
  );
}

export function createBoatAccountRuns(origin: string): readonly SmokeRun[] {
  return [
    {
      name: "overflow-rejected-disconnect",
      check: (page) => rejectedDisconnect(page, origin),
    },
    {
      name: "overflow-change-key-pending",
      check: (page) => changeKeyPending(page, origin),
    },
  ];
}
