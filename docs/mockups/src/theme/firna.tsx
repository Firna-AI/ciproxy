// The @firna/ui theme of the ciproxy dashboard and the static CSS that a
// server-rendered Firna control needs. The colour tokens are also published as
// CSS custom properties, so the mockup stylesheets read the same values.

import type { PropsWithChildren } from "react";
import { domBackendCss } from "@firna/ui";
import { focusRingCssVariablesFor } from "@firna/ui/focusRing";
import {
  createSharedUiTheme,
  SharedUiThemeProvider,
  type SharedUiTheme,
} from "@firna/ui/theme";

/** The dashboard theme: the Firna default palette with the bundled Inter face. */
export const theme = createSharedUiTheme({
  fonts: {
    sans: '"Inter Variable", Inter, system-ui, sans-serif',
    mono: '"Source Code Pro", "Liberation Mono", Menlo, Consolas, monospace',
  },
});

/** Emphasis charts: the accent bar and the grey of the context bars. */
export const emphasisColors = {
  accent: theme.colors.primary,
  /** 3.2:1 on white; the Firna faint grey is below 3:1. */
  context: "#8a918a",
} as const;

/** The two chart series, in the fixed order of the Firna categorical palette. */
export const seriesColors = {
  own: theme.charts.series[0] ?? theme.colors.primary,
  boat: theme.charts.series[1] ?? theme.colors.primaryDeep,
} as const;

/** The chart colours that a theme publishes beside its Firna tokens. */
export type ChartTokens = {
  own: string;
  boat: string;
  accent: string;
  context: string;
};

/** "primaryDeep" -> "primary-deep", the custom property spelling. */
export function kebab(name: string): string {
  return name.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`);
}

/** The tokens of a theme as `name:value` custom property declarations. */
export function themeDeclarations(
  source: SharedUiTheme,
  chart: ChartTokens,
): string[] {
  const colors = Object.entries(source.colors).map(
    ([name, value]) => `--c-${kebab(name)}:${value}`,
  );
  const radii = Object.entries(source.radii)
    .filter(([name]) => name !== "avatarRatio")
    .map(([name, value]) => `--r-${name}:${value}px`);
  const fonts = [
    `--font-sans:${source.fonts.sans}`,
    `--font-mono:${source.fonts.mono}`,
    `--c-series-own:${chart.own}`,
    `--c-series-boat:${chart.boat}`,
    `--c-chart-grid:${source.charts.grid}`,
    `--c-chart-deemphasis:${source.charts.deemphasis}`,
    `--c-chart-accent:${chart.accent}`,
    `--c-chart-context:${chart.context}`,
  ];
  const focus = Object.entries(
    focusRingCssVariablesFor(source.colors.primary),
  ).map(([name, value]) => `${name}:${value}`);
  return [...colors, ...radii, ...fonts, ...focus];
}

function tokenCss(): string {
  const chart = { ...seriesColors, ...emphasisColors };
  return `:root{${themeDeclarations(theme, chart).join(";")}}`;
}

const staticCss = `${domBackendCss}\n${tokenCss()}`;

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
