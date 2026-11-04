// The Graphite look of the dashboard: light and monochrome, in the manner of
// Vercel. Black is the one accent, colour marks status only, and the type is
// Geist with Geist Mono for technical values. Every colour of the catalogue
// comes from these tokens. theme/firna.tsx publishes them as CSS custom
// properties, so the product and the docs site can reuse the same values.

import "@fontsource-variable/geist/wght.css";
import "@fontsource-variable/geist-mono/wght.css";

import { createSharedUiTheme } from "@firna/ui/theme";

const ink = "#0A0A0A";

/** The Firna theme: the colours, faces, and radii of the Firna controls. */
export const theme = createSharedUiTheme({
  colors: {
    bg: "#FAFAFA",
    bg2: "#F4F4F4",
    surface: "#FFFFFF",
    soft: "#F4F4F4",
    border: "#EAEAEA",
    border2: "#DEDEDE",
    controlBorder: "rgba(10, 10, 10, 0.15)",
    ink,
    ink2: "#525252",
    /** 5.1:1 on white. */
    muted: "#6B6B6B",
    faint: "#A3A3A3",
    placeholder: "#737373",
    primary: ink,
    primaryDeep: ink,
    primarySoft: "#F0F0F0",
    primaryBorder: "#D4D4D4",
    onSolid: "#FFFFFF",
    amber: "#C27803",
    amberDeep: "#8A4B00",
    amberSoft: "#FFF5E0",
    rose: "#D93036",
    roseDeep: "#B4232A",
    roseSoft: "#FDECEC",
  },
  fonts: {
    sans: '"Geist Variable", Geist, system-ui, sans-serif',
    mono: '"Geist Mono Variable", "Geist Mono", ui-monospace, monospace',
  },
  radii: { sm: 4, md: 6, lg: 6, xl: 8, xxl: 8, pill: 999 },
});

/** The status colours. With black, they are the only colours on a screen. */
export const statusColors = {
  ok: "#12A150",
  warn: "#F5A524",
  err: "#E5484D",
  run: "#0070F3",
  idle: "#A3A3A3",
} as const;

export type StatusTone = keyof typeof statusColors;

/**
 * Chart colours: the two series of the starts chart, and the emphasis bars of
 * the cost chart. The context grey is 3.3:1 on white.
 */
export const chartColors = {
  own: "#171717",
  boat: "#A8A8A8",
  accent: ink,
  context: "#8F8F8F",
} as const;

/** The slots of a machine in the segmented bar. */
export const slotColors = {
  busy: "#1F1F1F",
  warm: "#A8A8A8",
  free: "#E8E8E8",
} as const;

/** The sidebar: labels between the ink and the muted grey, lighter icons. */
export const navColors = { label: "#5E5E5E", icon: "#8F8F8F" } as const;
