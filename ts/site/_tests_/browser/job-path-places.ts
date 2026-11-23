// The places of the boxes of the picture of How ciproxy works, as a reader
// sees them: a table of rows from top to bottom and columns from left to
// right. Boxes whose titles overlap from top to bottom share a row, and boxes
// whose titles overlap from left to right share a column. The browser tests
// of the docs compare the table with a literal table at two window widths.

import type { Page } from "playwright-core";

import { jobPath, type JobPathPart } from "../../src/data/job-path.ts";

/**
 * A cell of the table: "empty", the part of the box in it, or the parts of
 * two or more boxes in one cell, joined by "and".
 */
export type Cell = "empty" | JobPathPart | `${JobPathPart} and ${string}`;

/** A rectangle of the page, in CSS pixels, as Playwright gives it. */
type Rect = { x: number; y: number; width: number; height: number };

/** A box of the picture, and the rectangle of its title on the page. */
type Placed = { part: JobPathPart; rect: Rect };

/** The start and the end of a rectangle along one direction. */
type Extent = (rect: Rect) => { start: number; end: number };

const downwards: Extent = (rect) => ({
  start: rect.y,
  end: rect.y + rect.height,
});

const across: Extent = (rect) => ({ start: rect.x, end: rect.x + rect.width });

/** The boxes of the picture in reading order: the path, then Boat. */
const boxes = [...jobPath.steps, jobPath.boat];

/**
 * The parts of each line of boxes along one direction, in the order of that
 * direction. A box joins a line when it overlaps a box of that line.
 */
function lines(placed: readonly Placed[], extent: Extent): Set<JobPathPart>[] {
  const sorted = placed.toSorted(
    (a, b) => extent(a.rect).start - extent(b.rect).start,
  );
  const found: { end: number; parts: Set<JobPathPart> }[] = [];
  for (const box of sorted) {
    const { start, end } = extent(box.rect);
    const line = found.at(-1);
    if (line && start < line.end) {
      line.parts.add(box.part);
      line.end = Math.max(line.end, end);
    } else {
      found.push({ end, parts: new Set([box.part]) });
    }
  }
  return found.map((line) => line.parts);
}

/** The cell of the boxes in both this row and this column. */
function cell(row: Set<JobPathPart>, column: Set<JobPathPart>): Cell {
  const [first, ...more] = boxes
    .map((box) => box.part)
    .filter((part) => row.has(part) && column.has(part));
  if (first === undefined) return "empty";
  return more.length === 0 ? first : `${first} and ${more.join(" and ")}`;
}

/**
 * The table of the boxes of `jobPath` on a page that shows the picture. It
 * finds the picture by its caption and each box by its title, as a reader
 * does. A box that the page does not show has no place, so the table leaves
 * it out.
 */
export async function tableOfBoxes(page: Page): Promise<Cell[][]> {
  const figure = page.getByRole("figure", {
    name: jobPath.caption,
    exact: true,
  });
  await figure.waitFor();
  const placed: Placed[] = [];
  for (const { part, title } of boxes) {
    const rect = await figure.getByText(title, { exact: true }).boundingBox();
    if (rect) placed.push({ part, rect });
  }
  const columns = lines(placed, across);
  return lines(placed, downwards).map((row) =>
    columns.map((column) => cell(row, column)),
  );
}
