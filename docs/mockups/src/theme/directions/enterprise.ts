// Direction C, Enterprise: light content beside a deep navy sidebar, in the
// manner of Stripe and Datadog. One blue accent, white cards with a hairline
// and a faint shadow, and IBM Plex Sans with IBM Plex Mono.

import "@fontsource/ibm-plex-sans/latin-400.css";
import "@fontsource/ibm-plex-sans/latin-500.css";
import "@fontsource/ibm-plex-sans/latin-600.css";
import "@fontsource/ibm-plex-mono/latin-400.css";
import "@fontsource/ibm-plex-mono/latin-500.css";

import { createSharedUiTheme } from "@firna/ui/theme";

import type { Direction } from "../direction.ts";

const accent = "#2563EB";
const navy = "#0B1B33";

export const enterprise: Direction = {
  key: "enterprise",
  name: "Enterprise",
  firna: createSharedUiTheme({
    colors: {
      bg: "#F5F7FA",
      bg2: "#EEF1F5",
      surface: "#FFFFFF",
      soft: "#F1F4F8",
      border: "#E1E6ED",
      border2: "#D3DAE3",
      controlBorder: "rgba(15, 27, 45, 0.2)",
      ink: "#0F1B2D",
      ink2: "#3B4A5E",
      /** 5.4:1 on white. */
      muted: "#5A687C",
      faint: "#97A3B4",
      placeholder: "#6B7789",
      primary: accent,
      primaryDeep: "#1D4ED8",
      primarySoft: "#EAF1FE",
      primaryBorder: "#BFD3FB",
      onSolid: "#FFFFFF",
      amber: "#D97706",
      amberDeep: "#92400E",
      amberSoft: "#FEF3C7",
      rose: "#DC2626",
      roseDeep: "#991B1B",
      roseSoft: "#FEE2E2",
    },
    fonts: {
      sans: '"IBM Plex Sans", system-ui, sans-serif',
      mono: '"IBM Plex Mono", ui-monospace, monospace',
    },
    radii: { sm: 3, md: 4, lg: 4, xl: 4, xxl: 6, pill: 999 },
  }),
  status: {
    ok: "#16A34A",
    warn: "#F59E0B",
    err: "#DC2626",
    run: accent,
    idle: "#94A3B8",
  },
  series: { own: accent, boat: "#93C5FD" },
  /** The context grey is 2.6:1 on white; each bar also carries its value. */
  emphasis: { accent, context: "#94A3B8" },
  shell: "navy",
  kpi: "card",
  chart: "bars",
  vars: {
    bodySize: "13px",
    titleSize: "22px",
    rowHeight: "44px",
    sidebarBg: navy,
    sidebarBorder: navy,
    sidebarInk: "#FFFFFF",
    sidebarMuted: "#B4C1D6",
    sidebarIcon: "#8A9DBB",
    sidebarHover: "#12284A",
    sidebarActive: "#1C3760",
    topBg: navy,
    topInk: "#FFFFFF",
    topBorder: navy,
    headBg: "#F1F4F8",
    headInk: "#3B4A5E",
    shadow:
      "0 1px 2px rgba(15, 27, 45, 0.06), 0 1px 3px rgba(15, 27, 45, 0.04)",
    glow: "none",
    slotBusy: accent,
    slotWarm: "#A5C1F7",
    slotFree: "#E1E6ED",
  },
};
