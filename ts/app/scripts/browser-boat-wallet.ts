// The wallet journey of Settings under Overflow. Check key shows the wallets
// of the key with the likely wallet chosen, and focus stays on the button,
// which now reads Connect Boat. A key edit drops the wallets. The keyboard
// reaches the wallets, chooses the personal wallet, and connects it.
import assert from "node:assert/strict";

import {
  checkKey,
  checks,
  connections,
  focusedOn,
  harnessRecord,
  overflowPath,
} from "./browser-boat.ts";
import type { Page } from "./browser-driver.ts";
import { control, viewportControls, visible } from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import { fixtureBoatKey, refusedBoatKey } from "./fixtures-boat.ts";

/** Wait until focus rests on the wallet card with this name. */
async function walletFocused(page: Page, name: string): Promise<void> {
  await page.waitForFunction(
    (label) =>
      document.activeElement?.getAttribute("role") === "radio" &&
      document.activeElement.getAttribute("aria-label") === label,
    name,
  );
}

async function walletChoice(page: Page, origin: string, width: number) {
  await control(origin, { scenario: "overflow-not-connected" });
  await page.goto(origin + overflowPath, { waitUntil: "domcontentloaded" });
  await checkKey(page, "Boat API key", fixtureBoatKey, "150");
  const group = page.getByRole("radiogroup", {
    name: "Wallet that Boat bills",
  });
  await visible(group);
  await viewportControls(page, width);
  await visible(page.getByRole("radio", { name: "northstar", checked: true }));
  await visible(
    page.getByRole("radio", { name: "Mira Patel", checked: false }),
  );
  await focusedOn(page, "Connect Boat");
  await page.getByLabel("Boat API key", { exact: true }).fill(refusedBoatKey);
  await group.waitFor({ state: "hidden" });
  await visible(page.getByRole("button", { name: "Check key", exact: true }));
  await checkKey(page, "Boat API key", fixtureBoatKey, null);
  await visible(group);
  await focusedOn(page, "Connect Boat");
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Shift+Tab");
  await walletFocused(page, "Mira Patel");
  await page.keyboard.press("ArrowDown");
  await walletFocused(page, "northstar");
  await page.keyboard.press("ArrowUp");
  await walletFocused(page, "Mira Patel");
  await page.keyboard.press("Space");
  await visible(page.getByRole("radio", { name: "Mira Patel", checked: true }));
  await page.keyboard.press("Tab");
  await page.keyboard.press("Tab");
  await focusedOn(page, "Connect Boat");
  await page.keyboard.press("Enter");
  await visible(
    page.getByText("Connected to the account northstar", { exact: true }),
  );
  await focusedOn(page, "Boat");
  assert.deepEqual(await checks(origin), [200, 200]);
  assert.deepEqual(await connections(origin), [200]);
  assert.equal(
    await harnessRecord(origin, "boatConnectionWallet"),
    "personal",
    "Connect Boat sends the chosen wallet",
  );
}

export function createBoatWalletRuns(origin: string): readonly SmokeRun[] {
  return [
    {
      name: "overflow-wallet-choice",
      check: (page, _context, width) => walletChoice(page, origin, width),
    },
  ];
}
