// The CPU and memory panel of the job page in each of its states: the
// figures with a chart and the steps, the label size of a larger Boat size, a
// step that ran out of memory, a macOS job without that figure, a job without
// steps, a job of one hour, a running job, a job without figures, and a job
// that no runner took, which has no panel. Each check keeps the panel apart
// from the other two panels and inside the window. The checks count the
// drawn marks of the chart and the meters, which hide from screen readers,
// by their test IDs. The job screens of the page smoke run these checks, and
// three screens of their own open the out-of-memory, macOS, and one-hour
// jobs.

import assert from "node:assert/strict";

import { jobHref } from "../src/view/jobs-links.ts";
import type { Locator, Page } from "./browser-driver.ts";
import { insideViewport, visible } from "./browser-fixture.ts";
import {
  absent,
  boxInside,
  insideRegion,
  separateRegions,
  visibleBox,
  type Box,
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
 * The marks that the panel draws, by kind: the CPU and memory lines, the
 * line of the label size on a plot, the tick of the label size on a meter,
 * an out-of-memory dot, a step block of the lane, and the line at the start
 * of a step on a plot. A mark counts when its box lies inside the panel and
 * has a length. A level or an upright line has no height or no width, so one
 * of the two is enough.
 */
async function drawnMarks(panel: Locator) {
  const outer = await visibleBox(panel);
  const drawn = async (kind: string) => {
    let marks = 0;
    for (const mark of await panel.getByTestId(`usage-mark-${kind}`).all()) {
      const box = await mark.boundingBox();
      const length = box !== null && (box.width > 0 || box.height > 0);
      if (length && boxInside(outer, box)) marks += 1;
    }
    return marks;
  };
  return {
    lines: await drawn("line"),
    labelLines: await drawn("label-line"),
    labelTicks: await drawn("label-tick"),
    kills: await drawn("kill"),
    steps: await drawn("step"),
    starts: await drawn("start"),
  };
}

/** The rows of the steps table, and each step name that passes its cell. */
async function namesOutside(panel: Locator) {
  const table = panel.getByRole("table", { name: "Steps", exact: true });
  const [, ...rows] = await table.getByRole("row").all();
  const outside: (string | null)[] = [];
  for (const row of rows) {
    const [cell] = await row.getByRole("cell").all();
    assert.ok(cell !== undefined, "Each row starts with its Step cell.");
    const name = cell.getByTestId("usage-step-name");
    if (!boxInside(await visibleBox(cell), await visibleBox(name)))
      outside.push(await name.textContent());
  }
  return { rows: rows.length, outside };
}

/** The part of the panel that shows: the flush panel cuts off at its border. */
async function shownPart(panel: Locator): Promise<Box> {
  const box = await visibleBox(panel);
  const inner = await panel.evaluate((element) => ({
    x: element.clientLeft,
    y: element.clientTop,
    width: element.clientWidth,
    height: element.clientHeight,
  }));
  return { ...inner, x: box.x + inner.x, y: box.y + inner.y };
}

/**
 * The labels of the time axis, once the axis has its width. Each label must
 * show in full in the part of the panel that shows.
 */
async function timeLabels(panel: Locator): Promise<(string | null)[]> {
  await visible(panel.getByText("0:00", { exact: true }));
  const shown = await shownPart(panel);
  const labels: (string | null)[] = [];
  for (const label of await panel.getByTestId("usage-time").all()) {
    const text = await label.textContent();
    assert.ok(
      boxInside(shown, await visibleBox(label)),
      `The time ${text} must show in full in the panel.`,
    );
    labels.push(text);
  }
  return labels;
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
    const { steps, starts, labelLines, labelTicks } = await drawnMarks(panel);
    assert.deepEqual(
      { steps, starts, labelLines, labelTicks },
      { steps: 8, starts: 14, labelLines: 0, labelTicks: 0 },
      "a block for each step that lasted, its 7 later starts on both plots, and no label size",
    );
  }) satisfies UsageLayout,
  labelSize: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /The label gives 4\svCPU and 8\sGB\.$/);
    await inside(panel, "of 8 vCPU · label 4 vCPU");
    await inside(panel, "of 16 GB · label 8 GB");
    await inside(panel, "Label size: 4 vCPU · 8 GB");
    const { lines, labelLines, labelTicks } = await drawnMarks(panel);
    assert.deepEqual(
      { lines, labelLines, labelTicks },
      { lines: 2, labelLines: 2, labelTicks: 2 },
      "the CPU and memory lines, and the label size on each plot and meter",
    );
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
    assert.equal((await drawnMarks(panel)).kills, 1, "a dot for the one kill");
    assert.deepEqual(
      await namesOutside(panel),
      { rows: phone ? 0 : 9, outside: [] },
      "each step name, also a long word, stays inside its Step cell",
    );
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
  oneHour: (async (page, width) => {
    const panel = await placed(page, width);
    await figures(panel, /^CPU and memory over 1h 00m: /);
    assert.deepEqual(
      await timeLabels(panel),
      width === 390
        ? ["0:00", "15:00", "30:00", "45:00", "1:00:00"]
        : ["0:00", "10:00", "20:00", "30:00", "40:00", "50:00", "1:00:00"],
      "the axis names the end of the chart",
    );
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
  {
    name: "job-one-hour",
    scenario: "m21-jobs-detail",
    path: jobHref(workspaceId, fixtureJobIds.hour),
    heading: "Nightly end-to-end",
    layoutCheck: usageLayouts.oneHour,
  },
];
