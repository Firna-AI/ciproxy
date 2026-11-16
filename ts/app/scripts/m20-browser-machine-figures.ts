// The figures of Machines that a machine did not measure, in the browser: the
// machine page whose machine cache, Docker disks, or both are unavailable,
// for an Admin, a Billing person, and a Viewer, the Clear cache confirmation
// without a size, and the cache hits of the list with some or no measured
// machines. The figures come from the what-ifs of m20-fixtures-statistics.ts.

import assert from "node:assert/strict";

import type { Page } from "./browser-driver.ts";
import { visible } from "./browser-fixture.ts";
import {
  capture,
  dialogCancel,
  machinesUrl,
  open,
  phone,
  within,
} from "./m20-browser-fixture.ts";
import { forgeDocker } from "./m20-browser-machine-cache.ts";
import { machineIds } from "./m20-fixtures-machines.ts";
import type { M20Scenario } from "./m20-harness-state.ts";

const notes = {
  cache: "Cache disk and cache hits show after the machine measures them.",
  docker: "Docker disks show after the machine measures them.",
  both: "Cache disk, cache hits, and Docker disks show after the machine measures them.",
} as const;

/**
 * The Cache panel of forge-01: how many figures say Unknown, the note, and
 * whether the cache disk keeps its meter. A measured part keeps its figures.
 */
async function cacheFigures(
  page: Page,
  unknown: number,
  note: string,
  meter: boolean,
): Promise<void> {
  await visible(page.getByText(note, { exact: true }));
  assert.equal(
    await page.getByText("Unknown", { exact: true }).count(),
    unknown,
    "each unmeasured figure says Unknown",
  );
  assert.equal(
    await page.getByRole("progressbar", { name: "Cache disk" }).count(),
    meter ? 1 : 0,
    "an unknown cache disk has no meter",
  );
}

/** The figures of the machine page of forge-01 in one what-if. */
async function openForge(page: Page, origin: string, scenario: M20Scenario) {
  await open(
    page,
    origin,
    scenario,
    machinesUrl(origin, machineIds.primary),
    "forge-01",
  );
}

/**
 * An Admin sees each part of the cache apart: Unknown where the machine did
 * not measure it, and Clear cache, whose confirmation then names no size.
 */
export async function unknownFigures(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  await openForge(page, origin, "machine-cache-unknown");
  await cacheFigures(page, 2, notes.cache, false);
  await visible(page.getByText(forgeDocker));
  await capture(page, width, output, "forge-01-cache-unknown");
  await openForge(page, origin, "machine-docker-unknown");
  await cacheFigures(page, 1, notes.docker, true);
  await visible(page.getByText("96 of 402 GB", { exact: true }));
  await visible(page.getByText("84%", { exact: true }));
  await capture(page, width, output, "forge-01-docker-unknown");
  await openForge(page, origin, "machine-figures-unknown");
  await cacheFigures(page, 3, notes.both, false);
  await capture(page, width, output, "forge-01-figures-unknown");
  await page.getByRole("button", { name: "Clear cache", exact: true }).click();
  const title = "Clear the cache of forge-01?";
  const dialog = page.getByRole("dialog", { name: title, exact: true });
  await visible(
    within(dialog).getByText(
      "ciproxy deletes the cache entries on this machine. The GitHub cache keeps its entries, so the next jobs restore from there.",
      { exact: true },
    ),
  );
  await capture(page, width, output, "forge-01-figures-unknown-clear");
  await dialogCancel(page, title);
}

/** A Billing person and a Viewer read the same Unknown figures, with no action. */
export async function unknownFiguresReadOnly(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  for (const scenario of [
    "billing-figures-unknown",
    "viewer-figures-unknown",
  ] as const) {
    await openForge(page, origin, scenario);
    await cacheFigures(page, 3, notes.both, false);
    for (const action of ["Clear cache", "Drain"])
      assert.equal(
        await page.getByRole("button", { name: action, exact: true }).count(),
        0,
        `only an Admin gets ${action}`,
      );
    await capture(page, width, output, `forge-01-${scenario}`);
  }
}

/** The headline figure of Machines with this label: a group of that name. */
function figure(page: Page, label: string) {
  return page.getByRole("group", { name: label, exact: true });
}

/** The row of the machines table that this hostname leads. */
function machineRow(page: Page, hostname: string) {
  return page.getByRole("row", { name: new RegExp(`^${hostname}\\s`) });
}

/**
 * The cache hits of Machines cover the measured machines and name the
 * others: forge-01 measured no machine cache, and lab-no-kvm has no report.
 * With no measured machine, the figure says Unknown and names none.
 */
export async function cacheHitsFigure(
  page: Page,
  origin: string,
  width: number,
  output: string,
): Promise<void> {
  const label = "Cache hits, 7 days";
  await open(
    page,
    origin,
    "machine-cache-unknown",
    machinesUrl(origin),
    "Machines",
  );
  // The figures show with the first read, so their text is final once seen.
  const partial = figure(page, label);
  await visible(partial);
  assert.equal(
    await partial.textContent(),
    [label, "50%", "Excludes", "forge-01", "and", "lab-no-kvm"].join(""),
    "the share names the machines that it leaves out",
  );
  if (!phone(width)) {
    for (const [hostname, cache] of [
      ["forge-01", "Unknown"],
      ["lab-no-kvm", "No report yet"],
      ["office-mini-1", "6 of 117 GB"],
    ] as const)
      await visible(
        within(machineRow(page, hostname)).getByText(cache, { exact: true }),
      );
  }
  await capture(page, width, output, "machines-cache-unknown");
  await open(
    page,
    origin,
    "machines-no-cache-measured",
    machinesUrl(origin),
    "Machines",
  );
  const none = figure(page, label);
  await visible(none);
  assert.equal(await none.textContent(), `${label}Unknown`);
  assert.equal(await page.getByText("Excludes", { exact: true }).count(), 0);
  const unknown = within(none).getByText("Unknown", { exact: true });
  // The Machines figure has the same style, and its value takes one line.
  const oneLine = within(figure(page, "Machines")).getByText("7", {
    exact: true,
  });
  await visible(oneLine);
  const [unknownBox, oneLineBox] = [
    await unknown.boundingBox(),
    await oneLine.boundingBox(),
  ];
  assert.ok(unknownBox !== null && oneLineBox !== null);
  assert.equal(
    unknownBox.height,
    oneLineBox.height,
    "the word Unknown stays whole on one line",
  );
  await capture(page, width, output, "machines-no-cache-measured");
}
