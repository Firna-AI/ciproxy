import {
  chartColors,
  colors,
  fonts,
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

/** Global CSS for consumers that do not use Firna. */
export const tokenCss = `:root{${tokenDeclarations().join(";")}}`;
