// The ciproxy mark, after the brand of the mockups: two bars that meet as a
// chevron, one flow of jobs routed to an own machine or to overflow.
// BrandMark.astro draws these paths in the top bar, footer, and picture of
// How ciproxy works. The favicon draws them too.

import { colors } from "@ciproxy/tokens";

/** The paths of the mark, in a 24 by 24 box. */
export const brandMarkPaths = [
  "M2 3h8.6l8.4 8.1h-8.6z",
  "M10.4 12.9H19L10.6 21H2z",
] as const;

/**
 * The favicon: the mark in the ink, with no tile behind it. Its square box
 * fits the mark with half a unit of space, so the mark fills a small tab.
 */
export function brandFaviconSvg(): string {
  const paths = brandMarkPaths.map((d) => `<path d="${d}"/>`).join("");
  return (
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="1 2.5 19 19" ` +
    `fill="${colors.ink}">${paths}</svg>\n`
  );
}
