// The design directions of the dashboard: token sets that the shared
// components read inside a DirectionScope. Outside a scope, useDirection
// returns null and every component keeps the current Firna look.

import { createContext, useContext } from "react";
import type { SharedUiTheme } from "@firna/ui/theme";

import type { CataloguePath } from "../components/paths.ts";

export type DirectionKey = "graphite" | "midnight" | "enterprise";

/** The status colours. With the accent, they are the only colours. */
export type StatusTone = "ok" | "warn" | "err" | "run" | "idle";

/**
 * The chrome around a page: a white sidebar with a breadcrumb top bar, a dark
 * console sidebar, or a navy sidebar with a white page header band.
 */
export type ShellKind = "topbar" | "console" | "navy";

/**
 * The figures above a table: one divided strip, stat tiles with a faint area
 * trend behind the value, or cards with a trend line beside the value.
 */
export type KpiKind = "strip" | "stat" | "card";

/** The chart of starts by day: stacked bars, or thin lines over a faint area. */
export type ChartKind = "bars" | "lines";

/** Values that only the direction stylesheets read, as --d-* properties. */
export type DirectionVars = {
  bodySize: string;
  titleSize: string;
  rowHeight: string;
  sidebarBg: string;
  sidebarBorder: string;
  sidebarInk: string;
  sidebarMuted: string;
  sidebarIcon: string;
  sidebarHover: string;
  sidebarActive: string;
  topBg: string;
  topInk: string;
  topBorder: string;
  headBg: string;
  headInk: string;
  shadow: string;
  glow: string;
  slotBusy: string;
  slotWarm: string;
  slotFree: string;
};

export type Direction = {
  key: DirectionKey;
  name: string;
  /** The Firna theme: colours, fonts, radii, and scheme. */
  firna: SharedUiTheme;
  status: Readonly<Record<StatusTone, string>>;
  /** The two chart series: jobs on own machines and jobs on Boat. */
  series: { own: string; boat: string };
  /** Emphasis bars: the ciproxy bar and the grey of the other bars. */
  emphasis: { accent: string; context: string };
  shell: ShellKind;
  kpi: KpiKind;
  chart: ChartKind;
  vars: DirectionVars;
};

/** Links that open the direction's own copy of a screen, where one exists. */
export type DirectionRoutes = Readonly<
  Partial<Record<CataloguePath, CataloguePath>>
>;

type ScopeValue = { direction: Direction; routes: DirectionRoutes };

export const DirectionContext = createContext<ScopeValue | null>(null);

/** The direction of the nearest scope, or null for the current look. */
export function useDirection(): Direction | null {
  return useContext(DirectionContext)?.direction ?? null;
}

/** A resolver from a screen path to the path that a link opens in the scope. */
export function useRoutes(): (path: CataloguePath) => CataloguePath {
  const routes = useContext(DirectionContext)?.routes;
  return (path) => routes?.[path] ?? path;
}
