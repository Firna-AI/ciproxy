// The Firna boundary of the mockups: the static CSS that a server-rendered
// Firna control needs, and the Graphite tokens as CSS custom properties, so
// the stylesheets read the same values as the Firna controls.

import type { PropsWithChildren } from "react";
import { domBackendCss } from "@firna/ui";
import { focusRingCssVariablesFor } from "@firna/ui/focusRing";
import { SharedUiThemeProvider } from "@firna/ui/theme";

import "@ciproxy/tokens/fonts";
import {
  chartColors,
  navColors,
  slotColors,
  statusColors,
} from "@ciproxy/tokens";
import { theme } from "@ciproxy/tokens/firna";

/** "primaryDeep" -> "primary-deep", the custom property spelling. */
function kebab(name: string): string {
  return name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}

/** A token group as `--c-<group>-<name>:value` declarations. */
function colorGroup(
  group: string,
  values: Readonly<Record<string, string>>,
): string[] {
  const prefix = group ? `${group}-` : "";
  return Object.entries(values).map(
    ([name, value]) => `--c-${prefix}${kebab(name)}:${value}`,
  );
}

/** Every token as a custom property declaration. */
function tokenDeclarations(): string[] {
  const radii = Object.entries(theme.radii)
    .filter(([name]) => name !== "avatarRatio")
    .map(([name, value]) => `--r-${name}:${value}px`);
  const focus = Object.entries(
    focusRingCssVariablesFor(theme.colors.primary),
  ).map(([name, value]) => `${name}:${value}`);
  return [
    ...colorGroup("", theme.colors),
    ...colorGroup("status", statusColors),
    ...colorGroup("chart", chartColors),
    ...colorGroup("slot", slotColors),
    ...colorGroup("nav", navColors),
    ...radii,
    `--font-sans:${theme.fonts.sans}`,
    `--font-mono:${theme.fonts.mono}`,
    ...focus,
  ];
}

const staticCss = `${domBackendCss}\n:root{${tokenDeclarations().join(";")}}`;

/** The Firna DOM backend rules and the theme tokens, collected into the head. */
export function FirnaStaticStyle() {
  return <style data-firna-static-css="true">{staticCss}</style>;
}

/** The themed, boxless Firna boundary around the rendered screen. */
export function FirnaThemeProvider({ children }: PropsWithChildren) {
  return (
    <SharedUiThemeProvider theme={theme}>{children}</SharedUiThemeProvider>
  );
}
