import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { facts, visible } from "./browser-fixture.ts";
import { machineIds } from "./m20-fixtures-machines.ts";
import {
  capture,
  dialogCancel,
  fill,
  m20Control,
  machinesUrl,
  open,
  within,
} from "./m20-browser-fixture.ts";

export async function machineControls(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await open(
    page,
    origin,
    "machines",
    machinesUrl(origin, machineIds.primary),
    "forge-01",
  );
  await m20Control(origin, "machines-load-failure", false);
  await page.clock.runFor(10001);
  const refreshFailure = page.getByText("Unable to refresh machines", {
    exact: true,
  });
  await visible(refreshFailure);
  await m20Control(origin, "machines", false);
  await page.clock.runFor(10001);
  await refreshFailure.waitFor({ state: "hidden" });
  await page.getByRole("button", { name: "Drain", exact: true }).click();
  await visible(
    page.getByRole("button", { name: "Stop draining", exact: true }),
  );
  await page
    .getByRole("button", { name: "Stop draining", exact: true })
    .click();
  await visible(page.getByRole("button", { name: "Drain", exact: true }));
  await page
    .getByRole("button", { name: "More warm runners", exact: true })
    .click();
  await visible(
    page.getByRole("button", { name: "Fewer warm runners", exact: true }),
  );
  await page.getByRole("button", { name: "Edit", exact: true }).click();
  await fill(page, "Monthly cost in US dollars", "0");
  const saved = page.waitForResponse(/\/machines\/[^/]+\/cost$/);
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await (await saved).finished();
  await page
    .getByRole("textbox", { name: "Monthly cost in US dollars", exact: true })
    .waitFor({ state: "hidden" });
  const clear = page.getByRole("button", { name: "Clear cache", exact: true });
  await clear.click();
  await capture(page, width, output, "machine-clear-cache-dialog");
  await dialogCancel(page, "Clear the cache of forge-01?");
  // Enter reopens the dialog only when Cancel restores focus to its trigger.
  await page.keyboard.press("Enter");
  const dialog = page.getByRole("dialog", {
    name: "Clear the cache of forge-01?",
    exact: true,
  });
  await visible(dialog);
  await within(dialog)
    .getByRole("button", { name: "Clear cache", exact: true })
    .click();
  await dialog.waitFor({ state: "hidden" });
  await capture(page, width, output, "machine-controls-saved");
}

export async function addMachine(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await open(
    page,
    origin,
    "add-machine-waiting",
    machinesUrl(origin, undefined, true),
    "Add a machine",
  );
  await visible(
    page.getByRole("button", { name: "Revoke token", exact: true }),
  );
  await visible(
    page.getByText("Waiting for the machine to connect…", { exact: true }),
  );
  await capture(page, width, output, "add-machine-waiting");
  await m20Control(origin, "machines-load-failure", false);
  await page.clock.runFor(5001);
  const refreshFailure = page.getByText("Unable to refresh machines", {
    exact: true,
  });
  await visible(refreshFailure);
  await m20Control(origin, "add-machine-waiting", false);
  await page.clock.runFor(5001);
  await refreshFailure.waitFor({ state: "hidden" });
  assert.equal(
    (await facts(origin)).filter(
      (fact) => fact.method === "POST" && fact.path.endsWith("/install-tokens"),
    ).length,
    1,
    "A machines refresh must preserve the current install command",
  );
  await page.getByRole("button", { name: "Revoke token", exact: true }).click();
  await visible(page.getByText("Token revoked", { exact: true }));
  await capture(page, width, output, "add-machine-token-revoked");
  await page
    .getByRole("button", { name: "Create a new command", exact: true })
    .click();
  await visible(
    page.getByRole("button", { name: "Revoke token", exact: true }),
  );
  await m20Control(origin, "add-machine-connected", false);
  await page.clock.runFor(5001);
  await visible(page.getByText("forge-01 connected", { exact: true }));
  await visible(page.getByText("office-mini-1 connected", { exact: true }));
  await visible(
    page.getByText(/^Its disk holds 3\sjobs of ciproxy at a time\.$/),
  );
  await capture(page, width, output, "add-machine-connected");
  await page.getByRole("link", { name: "Open Machines", exact: true }).click();
  await page.waitForURL(machinesUrl(origin));
  await visible(page.getByRole("heading", { name: "Machines", exact: true }));
}

export async function machineFailure(
  page: Page,
  origin: string,
): Promise<void> {
  await open(
    page,
    origin,
    "machine-mutation-failure",
    machinesUrl(origin, machineIds.primary),
    "forge-01",
  );
  await page.getByRole("button", { name: "Drain", exact: true }).click();
  await visible(page.getByText("Unable to drain forge-01", { exact: true }));
  await visible(page.getByRole("button", { name: "Drain", exact: true }));
}
