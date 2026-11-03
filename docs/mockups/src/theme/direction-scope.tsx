// The scope of a design direction around a screen. It publishes the tokens of
// the direction as custom properties on its own class, so the stylesheets
// read them, nests the direction's Firna theme, and tells the shared
// components which direction to draw.

import type { ReactNode } from "react";
import { SharedUiThemeProvider } from "@firna/ui/theme";

import {
  DirectionContext,
  type Direction,
  type DirectionRoutes,
} from "./direction.ts";
import { kebab, themeDeclarations } from "./firna.tsx";
import "./direction-base.css";

function scopeCss(direction: Direction): string {
  const tokens = themeDeclarations(direction.firna, {
    ...direction.series,
    ...direction.emphasis,
  });
  const vars = Object.entries(direction.vars).map(
    ([name, value]) => `--d-${kebab(name)}:${value}`,
  );
  const status = Object.entries(direction.status).map(
    ([tone, value]) => `--d-${tone}:${value}`,
  );
  return `.dir-${direction.key}{${[...tokens, ...vars, ...status].join(";")}}`;
}

export function DirectionScope({
  direction,
  routes = {},
  children,
}: {
  direction: Direction;
  /** Links that open the direction's own copy of a screen. */
  routes?: DirectionRoutes;
  children: ReactNode;
}) {
  return (
    <>
      <style>{scopeCss(direction)}</style>
      <SharedUiThemeProvider theme={direction.firna}>
        <DirectionContext.Provider value={{ direction, routes }}>
          <div className={`dir dir-${direction.key}`}>{children}</div>
        </DirectionContext.Provider>
      </SharedUiThemeProvider>
    </>
  );
}
