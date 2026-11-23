// The places of the boxes of the picture of How ciproxy works, as a reader
// sees them: each box from the box before it. The browser tests of the docs
// read them at two window widths.

import type { Page } from "playwright-core";

import { jobPath } from "../../src/data/job-path.ts";

/** The place of a box of the picture, from the box before it. */
export type Place = "right" | "below" | "elsewhere";

/** A rectangle of the page, in CSS pixels, as Playwright gives it. */
type Rect = { x: number; y: number; width: number; height: number };

/**
 * "right" for a box in the row of the box before it, after its right edge.
 * "below" for a box in the column of the box before it, under its bottom.
 */
function placeAfter(previous: Rect, box: Rect): Place {
  const sameRow =
    box.y < previous.y + previous.height && previous.y < box.y + box.height;
  const sameColumn =
    box.x < previous.x + previous.width && previous.x < box.x + box.width;
  if (sameRow && box.x >= previous.x + previous.width) return "right";
  if (sameColumn && box.y >= previous.y + previous.height) return "below";
  return "elsewhere";
}

/**
 * The place of each box with these titles, from the box before it, on a page
 * that shows the picture. It finds the picture by its caption and each box by
 * its title, as a reader does.
 */
export async function placesOfBoxes(
  page: Page,
  titles: readonly string[],
): Promise<Place[]> {
  const figure = page.getByRole("figure", {
    name: jobPath.caption,
    exact: true,
  });
  await figure.waitFor();
  const places: Place[] = [];
  let previous: Rect | undefined;
  for (const title of titles) {
    const box = await figure.getByText(title, { exact: true }).boundingBox();
    if (!box) throw new Error(`The picture shows no box "${title}"`);
    if (previous) places.push(placeAfter(previous, box));
    previous = box;
  }
  return places;
}
