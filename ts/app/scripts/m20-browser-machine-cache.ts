// The Cache panel of the machine page in the browser: the Docker disks of the
// last report in each machine state and role, a report of none, a machine
// that has not reported yet, and the first read of the page while it waits
// and when it fails. The facts come from the fixture machines of the
// Machines harness.

import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import { fixtureTime } from "./fixtures-core.ts";
import {
  m20Control,
  m20HoldMachines,
  machinesUrl,
  open,
} from "./m20-browser-fixture.ts";
import { machineIds } from "./m20-fixtures-machines.ts";

/** forge-01 reports a Docker disk for 3 repositories, in 18 GB. */
export const forgeDocker = /^3\srepositories\s·\s18\sGB$/;

/** office-mini-1 reports that it holds no Docker disk. */
export const noDocker = /^0\srepositories\s·\s0\sGB$/;

/**
 * The Docker disks of a machine page: the count and the space of the last
 * report, or, before the first report, the cache note and no Docker disks.
 */
export async function dockerDisks(
  page: Page,
  hostname: string,
  docker: RegExp | null,
): Promise<void> {
  if (docker !== null) {
    await visible(page.getByText(docker));
    return;
  }
  await visible(
    page.getByText("Cache use shows after the machine reports it.", {
      exact: true,
    }),
  );
  assert.equal(
    await page.getByText("Docker disks", { exact: true }).count(),
    0,
    `${hostname} has no report, so it shows no Docker disks`,
  );
}

/** A Viewer or a Billing person reads the Docker disks, with no Clear cache. */
export async function readOnlyCache(page: Page): Promise<void> {
  await visible(page.getByText(forgeDocker));
  assert.equal(
    await page
      .getByRole("button", { name: "Clear cache", exact: true })
      .count(),
    0,
    "Only an Admin clears the cache",
  );
}

/**
 * The first read of the machine page. While it waits, the page shows its
 * loading line and no Docker disks. The harness holds that read until the
 * check releases it. When the first read of a new document fails, the page
 * shows the failure with Try again and no Docker disks. Try again then shows
 * the real row. The clock stays paused, so no poll reads the machines.
 */
export async function machineReadStates(
  page: Page,
  origin: string,
): Promise<void> {
  await page.clock.pauseAt(new Date(Date.parse(fixtureTime) + 5_000));
  await m20Control(origin, "machines");
  await m20HoldMachines(origin, true);
  try {
    const read = page.waitForRequest(/\/api\/workspaces\/[^/]+\/machines$/);
    await page.goto(machinesUrl(origin, machineIds.primary), {
      waitUntil: "domcontentloaded",
    });
    await read;
    await visible(page.getByText("Loading machines…", { exact: true }));
    assert.equal(
      await page.getByText("Docker disks", { exact: true }).count(),
      0,
      "The page shows no Docker disks before the first read",
    );
  } finally {
    await m20HoldMachines(origin, false);
  }
  const heading = page.getByRole("heading", { name: "forge-01", exact: true });
  await visible(heading);
  await visible(page.getByText(forgeDocker));
  await open(
    page,
    origin,
    "machines-load-failure",
    machinesUrl(origin, machineIds.primary),
    "Unable to load machines",
  );
  const retry = page.getByRole("button", { name: "Try again", exact: true });
  await visible(retry);
  assert.equal(
    await page.getByText("Docker disks", { exact: true }).count(),
    0,
    "A failed read shows no Docker disks",
  );
  await m20Control(origin, "machines", false);
  await retry.click();
  await visible(heading);
  await visible(page.getByText(forgeDocker));
}
