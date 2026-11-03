// Direction B, Midnight: a dark infrastructure console, in the manner of
// Linear and Grafana. One blue accent, status colours close to GitHub dark,
// and Inter with JetBrains Mono. Live states carry a dot with a soft glow.

import "@fontsource/jetbrains-mono/latin-400.css";
import "@fontsource/jetbrains-mono/latin-500.css";

import { createSharedUiTheme, darkSharedUiTheme } from "@firna/ui/theme";

import type { Direction } from "../direction.ts";

const accent = "#5B8CFF";

export const midnight: Direction = {
  key: "midnight",
  name: "Midnight",
  firna: createSharedUiTheme(
    {
      colors: {
        bg: "#0A0C10",
        bg2: "#0E1116",
        surface: "#11141A",
        soft: "#161A21",
        border: "#222834",
        border2: "#2C3340",
        controlBorder: "rgba(230, 232, 235, 0.16)",
        ink: "#E6E8EB",
        ink2: "#C2C8D1",
        muted: "#9BA3AF",
        faint: "#5D6573",
        placeholder: "#8A93A1",
        /** The fill of solid controls: 4.9:1 under white text. */
        primary: "#3A69E0",
        primaryDeep: accent,
        primarySoft: "#141E35",
        primaryBorder: "#2A4170",
        onSolid: "#FFFFFF",
        amber: "#D29922",
        amberDeep: "#E3B341",
        amberSoft: "#2A2112",
        rose: "#F85149",
        roseDeep: "#FF7B72",
        roseSoft: "#301819",
      },
      fonts: {
        sans: '"Inter Variable", Inter, system-ui, sans-serif',
        mono: '"JetBrains Mono", ui-monospace, monospace',
      },
      radii: { sm: 4, md: 6, lg: 6, xl: 6, xxl: 8, pill: 999 },
    },
    darkSharedUiTheme,
  ),
  status: {
    ok: "#3FB950",
    warn: "#D29922",
    err: "#F85149",
    run: "#58A6FF",
    idle: "#8B949E",
  },
  series: { own: accent, boat: "#8B949E" },
  /** The context grey is 3.0:1 on the panel. */
  emphasis: { accent, context: "#5C6573" },
  shell: "console",
  kpi: "stat",
  chart: "lines",
  vars: {
    bodySize: "13px",
    titleSize: "20px",
    rowHeight: "40px",
    sidebarBg: "#0A0C10",
    sidebarBorder: "#1A1F27",
    sidebarInk: "#E6E8EB",
    sidebarMuted: "#9BA3AF",
    sidebarIcon: "#7D8590",
    sidebarHover: "#12161C",
    sidebarActive: "#181D26",
    topBg: "#0A0C10",
    topInk: "#E6E8EB",
    topBorder: "#1A1F27",
    headBg: "#0E1116",
    headInk: "#8B93A1",
    shadow: "none",
    glow: "0 0 0 3px color-mix(in srgb, currentColor 16%, transparent), 0 0 10px 1px color-mix(in srgb, currentColor 55%, transparent)",
    slotBusy: accent,
    slotWarm: "#2B3E66",
    slotFree: "#1C212A",
  },
};
