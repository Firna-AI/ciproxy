// The Firna boundary of the mockups: the static CSS that a server-rendered
// Firna control needs, and the Graphite tokens as CSS custom properties, so
// the stylesheets read the same values as the Firna controls.

import type { PropsWithChildren } from "react";
import { domBackendCss } from "@firna/ui";
import { focusRingCssVariablesFor } from "@firna/ui/focusRing";
import { SharedUiThemeProvider } from "@firna/ui/theme";

import "@ciproxy/tokens/fonts";
import { tokenDeclarations } from "@ciproxy/tokens/css";
import { theme } from "@ciproxy/tokens/firna";

const focusDeclarations = Object.entries(
  focusRingCssVariablesFor(theme.colors.primary),
).map(([name, value]) => `${name}:${value}`);
const declarations = [...tokenDeclarations(), ...focusDeclarations];
const staticCss = `${domBackendCss}\n:root{${declarations.join(";")}}`;

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
