import {
  chartColors,
  colors,
  fonts,
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

/** Plain custom properties, with the same order as the Firna theme. */
export function tokenDeclarations(): string[] {
  const sortedColors = Object.fromEntries(
    Object.entries(colors).sort(([left], [right]) => left.localeCompare(right)),
  );
  const radiusDeclarations = Object.entries(radii)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([name, value]) => `--r-${name}:${value}px`);
  return [
    ...colorDeclarations("", sortedColors),
    ...colorDeclarations("status", statusColors),
    ...colorDeclarations("chart", chartColors),
    ...colorDeclarations("slot", slotColors),
    ...colorDeclarations("nav", navColors),
    ...radiusDeclarations,
    `--font-sans:${fonts.sans}`,
    `--font-mono:${fonts.mono}`,
  ];
}

/** Global CSS for consumers that do not use Firna. */
export const tokenCss = `:root{${tokenDeclarations().join(";")}}`;
