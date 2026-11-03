// The Mokly renderer of the ciproxy dashboard mockups. Mokly calls it around
// every React screen and owns the document; the renderer adds the Firna theme,
// the static Firna CSS, the Inter face, and the base stylesheet.

import "@fontsource-variable/inter/index.css";
import "./styles/base.css";

import type { RenderInput, RenderResult } from "@mokly/mokly";

import { FirnaStaticStyle, FirnaThemeProvider } from "./theme/firna.tsx";

export default function render(input: RenderInput): RenderResult {
  return {
    node: (
      <>
        <FirnaStaticStyle />
        <FirnaThemeProvider>
          <div className={`mockup mockup-${input.viewport}`} lang="en">
            {input.node}
          </div>
        </FirnaThemeProvider>
      </>
    ),
  };
}
