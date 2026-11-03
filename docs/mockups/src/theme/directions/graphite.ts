// Direction A, Graphite: light and monochrome, in the manner of Vercel. Black
// is the one accent, colour appears only for status, and the type is Geist
// with Geist Mono for technical values.

import "@fontsource-variable/geist/wght.css";
import "@fontsource-variable/geist-mono/wght.css";

import { createSharedUiTheme } from "@firna/ui/theme";

import type { Direction } from "../direction.ts";

const ink = "#0A0A0A";

export const graphite: Direction = {
  key: "graphite",
  name: "Graphite",
  firna: createSharedUiTheme({
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
  }),
  status: {
    ok: "#12A150",
    warn: "#F5A524",
    err: "#E5484D",
    run: "#0070F3",
    idle: "#A3A3A3",
  },
  series: { own: "#171717", boat: "#A8A8A8" },
  /** The context grey is 3.3:1 on white. */
  emphasis: { accent: ink, context: "#8F8F8F" },
  shell: "topbar",
  kpi: "strip",
  chart: "bars",
  vars: {
    bodySize: "14px",
    titleSize: "24px",
    rowHeight: "44px",
    sidebarBg: "#FFFFFF",
    sidebarBorder: "#EAEAEA",
    sidebarInk: ink,
    sidebarMuted: "#5E5E5E",
    sidebarIcon: "#8F8F8F",
    sidebarHover: "#F5F5F5",
    sidebarActive: "#EFEFEF",
    topBg: "#FFFFFF",
    topInk: ink,
    topBorder: "#EAEAEA",
    headBg: "#FAFAFA",
    headInk: "#6B6B6B",
    shadow: "none",
    glow: "none",
    slotBusy: "#1F1F1F",
    slotWarm: "#A8A8A8",
    slotFree: "#E8E8E8",
  },
};
