// The Setup states of a machine that gets its first job image, in the
// browser. The machine step shows the image line when the machine joins, and
// its closed line names the image until the machine is ready. After Skip for
// now, a first job that waits for a runner says that it waits for the image.

import { resolve } from "node:path";

import type { Page } from "./browser-driver.ts";
import { control, viewportControls, visible } from "./browser-fixture.ts";
import type { SmokeRun } from "./browser-registry.ts";
import { workspaceId } from "./fixtures-core.ts";

const setup = `/setup?workspace=${workspaceId}`;
const imageLine = "Getting the job image. Jobs start on it when it is ready.";

async function capture(
  page: Page,
  width: number,
  output: string,
  name: string,
): Promise<void> {
  await viewportControls(page, width);
  await page.screenshot({
    path: resolve(output, `${name}-${width}.png`),
    fullPage: true,
  });
}

/** The machine joins while Setup is open, then gets its first job image. */
async function machinePreparing(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await control(origin, { scenario: "setup-waiting" });
  await page.goto(origin + setup, { waitUntil: "domcontentloaded" });
  await visible(
    page.getByText("Waiting for the machine to connect…", { exact: true }),
  );
  await control(origin, { scenario: "setup-preparing", reset: false });
  await page.clock.runFor(5001);
  await visible(page.getByText("forge-01 connected", { exact: true }));
  await visible(page.getByText(imageLine, { exact: true }));
  await capture(page, width, output, "setup-machine-preparing-open");
  await page.getByRole("button", { name: "Continue", exact: true }).click();
  const closed = page.getByText("forge-01 · getting the job image", {
    exact: true,
  });
  await visible(closed);
  await capture(page, width, output, "setup-machine-preparing-closed");
  await control(origin, { scenario: "setup-connected", reset: false });
  await page.clock.runFor(5001);
  await visible(page.getByText("forge-01 · 6 slots", { exact: true }));
  await closed.waitFor({ state: "hidden" });
}

/** With Boat skipped, the first job waits for the image of the machine. */
async function firstJobWaits(page: Page, origin: string): Promise<void> {
  await control(origin, { scenario: "setup-image-wait" });
  await page.goto(origin + setup, { waitUntil: "domcontentloaded" });
  await visible(
    page.getByText("forge-01 · getting the job image", { exact: true }),
  );
  await page.getByRole("button", { name: "Skip for now", exact: true }).click();
  await visible(
    page.getByText(
      "Build and test in northstar-labs/control is waiting for a runner…",
      { exact: true },
    ),
  );
  await visible(
    page.getByText("forge-01 is getting the job image", { exact: true }),
  );
  await visible(
    page.getByText("The job starts on it when it is ready.", { exact: true }),
  );
}

export function createImageRuns(origin: string, output: string): SmokeRun[] {
  return [
    {
      name: "setup-machine-preparing",
      check: (page, _context, width) =>
        machinePreparing(page, origin, width, output),
    },
    {
      name: "setup-image-wait",
      check: (page) => firstJobWaits(page, origin),
    },
  ];
}
