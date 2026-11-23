// The CPU and memory panel of the job page in each of its states: the
// figures with a chart and the steps, the label size of a larger Boat size, a
// step that ran out of memory, a macOS job without that figure, a job without
// steps, a running job, a job without figures, and a job that no runner took,
// which has no panel. Each check keeps the panel apart from the other two
// panels and inside the window. The job screens of the page smoke run these
// checks, and two screens of their own open the out-of-memory and macOS jobs.

import assert from "node:assert/strict";

import { jobHref } from "../src/view/jobs-links.ts";
import type { Locator, Page } from "./browser-driver.ts";
import { insideViewport, visible } from "./browser-fixture.ts";
import {
  absent,
  insideRegion,
  separateRegions,
} from "./browser-pages-helpers.ts";
import type { PageScreen } from "./browser-pages-screens.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fixtureJobIds } from "./fixtures-jobs.ts";

type UsageLayout = (page: Page, width: number) => Promise<void>;

function usagePanel(page: Page): Locator {
  return page.getByRole("region", { name: "CPU and memory", exact: true });
}

/** The panel lies apart from the other two panels, inside the window. */
async function placed(page: Page, width: number): Promise<Locator> {
  const panel = usagePanel(page);
  for (const name of ["Where it ran", "Details"])
    await separateRegions(
      panel,
      page.getByRole("region", { name, exact: true }),
    );
  await insideViewport(panel, width);
  return panel;
}

/** The meters of the job and the chart, which names its span. */
async function figures(panel: Locator, summary: RegExp): Promise<void> {
  for (const name of ["Average CPU", "Peak memory"])
    await visible(panel.getByRole("progressbar", { name, exact: true }));
  await insideRegion(panel, panel.getByRole("figure", { name: summary }));
}

async function inside(panel: Locator, words: string): Promise<void> {
  await insideRegion(panel, panel.getByText(words, { exact: true }));
}

async function count(panel: Locator, words: string): Promise<number> {
  return panel.getByText(words, { exact: true }).count();
}

/**
 * The nine steps as a screen reader moves through them: a list on a phone,
 * and on a wider window a table with a header row and a row for each step,
 * with a cell under each of the four column headers.
 */
async function stepRoles(panel: Locator, phone: boolean): Promise<void> {
  const table = panel.getByRole("table", { name: "Steps", exact: true });
  const headers = await table.getByRole("columnheader").all();
  assert.deepEqual(
    {
      tables: await table.count(),
      rows: await table.getByRole("row").count(),
      headers: await Promise.all(headers.map((header) => header.textContent())),
      cells: await table.getByRole("cell").count(),
      items: await panel
        .getByRole("list", { name: "Steps", exact: true })
        .getByRole("listitem")
        .count(),
    },
    phone
      ? { tables: 0, rows: 0, headers: [], cells: 0, items: 9 }
      : {
          tables: 1,
          rows: 10,
          headers: ["Step", "Duration", "Average CPU", "Peak memory"],
          cells: 36,
          items: 0,
        },
    "the steps are a list on a phone, and a table on a wider window",
  );
}

/** One sentence in place of the figures. */
function sentence(words: string): UsageLayout {
  return async (page, width) => {
    const panel = await placed(page, width);
    await inside(panel, words);
    assert.equal(await panel.getByRole("figure").count(), 0);
  };
}

export const usageLayouts = {
  steps: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /^CPU and memory over 6m 12s: /);
    await inside(
      panel,
      "Each figure covers the whole VM, also the services and processes that earlier steps started.",
    );
    const script = panel.getByRole("link", {
      name: "Run script, opens step 4 on GitHub",
      exact: true,
    });
    await insideRegion(panel, script);
    assert.equal(
      await script.getAttribute("href"),
      "https://github.com/northstar-labs/control/actions/runs/701/job/801#step:4:1",
    );
    assert.equal(await script.getAttribute("target"), "_blank");
    assert.equal(
      await count(panel, "Out of memory"),
      1,
      "without a kill, only the figure of the job names out of memory",
    );
    await stepRoles(panel, width === 390);
  }) satisfies UsageLayout,
  labelSize: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /The label gives 4\svCPU and 8\sGB\.$/);
    await inside(panel, "of 8 vCPU · label 4 vCPU");
    await inside(panel, "of 16 GB · label 8 GB");
    await inside(panel, "Label size: 4 vCPU · 8 GB");
  }) satisfies UsageLayout,
  outOfMemory: (async (page, width) => {
    const panel = await placed(page, width);
    const phone = width === 390;
    await figures(panel, /^CPU and memory over 6m 12s: /);
    await inside(panel, "Skipped");
    assert.deepEqual(
      [await count(panel, "Out of memory"), await count(panel, "1 process")],
      phone ? [2, 1] : [3, 2],
      "the figure, the key, and on a wider window the column of the step",
    );
    assert.equal(await count(panel, "Out of memory: 1 process"), phone ? 1 : 0);
  }) satisfies UsageLayout,
  macos: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /^CPU and memory over 6m 12s: /);
    await inside(panel, "Complete job");
    assert.equal(await count(panel, "Out of memory"), 0);
  }) satisfies UsageLayout,
  noSteps: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /^CPU and memory over 20m 34s: /);
    await inside(
      panel,
      "Figures for each step are not available for this job.",
    );
    assert.equal(await count(panel, "Step"), 0, "no lane and no table");
  }) satisfies UsageLayout,
  noFigures: sentence("Figures are not available for this job."),
  running: sentence("The figures show when the job ends."),
  noRunner: (async (page) => {
    await absent(usagePanel(page));
  }) satisfies UsageLayout,
};

/** The jobs whose states no other job screen opens. */
export const usageScreens: PageScreen[] = [
  {
    name: "job-out-of-memory",
    scenario: "m21-jobs-detail",
    path: jobHref(workspaceId, fixtureJobIds.failed),
    heading: "Lint",
    layoutCheck: usageLayouts.outOfMemory,
  },
  {
    name: "job-macos",
    scenario: "m21-jobs-detail",
    path: jobHref(workspaceId, fixtureJobIds.older),
    heading: "Older retained job",
    layoutCheck: usageLayouts.macos,
  },
];
