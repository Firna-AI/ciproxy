import {
  chartColors,
  colors,
  fonts,
  heroStudyColors,
  markRadii,
  navColors,
  radii,
  slotColors,
  statusColors,
} from "./index.ts";

/** A token group as `--c-<group>-<name>:value` declarations. */
export function colorDeclarations(
  group: string,
  values: Readonly<Record<string, string>>,
): string[] {
  const prefix = group ? `${group}-` : "";
  return Object.entries(values).map(
    ([name, value]) =>
      `--c-${prefix}${name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}:${value}`,
  );
}

/** A radius group as `--r-<group>-<name>:<value>px` declarations. */
function radiusDeclarations(
  group: string,
  values: Readonly<Record<string, number>>,
): string[] {
  const prefix = group ? `${group}-` : "";
  return Object.entries(values).map(
    ([name, value]) => `--r-${prefix}${name}:${value}px`,
  );
}

/** The shared tokens as CSS custom property declarations. */
export function tokenDeclarations(): string[] {
  return [
    ...colorDeclarations("", colors),
    ...colorDeclarations("hero", heroStudyColors),
    ...colorDeclarations("status", statusColors),
    ...colorDeclarations("chart", chartColors),
    ...colorDeclarations("slot", slotColors),
    ...colorDeclarations("nav", navColors),
    ...radiusDeclarations("", radii),
    ...radiusDeclarations("mark", markRadii),
    `--font-sans:${fonts.sans}`,
    `--font-mono:${fonts.mono}`,
  ];
}

/**
 * `@font-face` rules for the variable Graphite faces, keyed by family name.
 *
 * Each face declares the full weight range of its variable file. Without the
 * range, a browser takes the face as weight 400 only. Safari then adds a
 * synthetic bold on top of the real bold weight, and text over 500 shows too
 * heavy. The range matches the Fontsource CSS that the site loads.
 */
export function fontFaceCss(urls: Readonly<Record<string, string>>): string {
  return Object.entries(urls)
    .map(
      ([family, url]) =>
        `@font-face{font-family:${JSON.stringify(family)};` +
        `src:url(${JSON.stringify(url)}) format("woff2");` +
        `font-weight:${fontWeightRange};font-style:normal;font-display:auto}`,
    )
    .join("");
}

/** The weight axis of the Geist and Geist Mono variable files. */
const fontWeightRange = "100 900";

/** Global CSS for consumers that do not use Firna. */
export const tokenCss = `:root{${tokenDeclarations().join(";")}}`;
