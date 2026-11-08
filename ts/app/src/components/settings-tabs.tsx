// The tabs of Settings, as the mockups show them: a row of links over a
// hairline, with the current tab in ink and underlined. Each tab is a real
// link that the router opens in place. Labels is the Settings page, and
// Overflow has its own page. The other tabs of the mockups join this list
// with their pages.

import { useState, type MouseEvent } from "react";
import { Link } from "expo-router";
import { useFocusRing } from "@firna/ui/focusRing";

import { colors, fonts } from "@ciproxy/tokens";

import { overflowHref, workspaceHref } from "../routing/pages.ts";

export type SettingsTab = "labels" | "overflow";

const tabs: readonly {
  key: SettingsTab;
  label: string;
  href: (workspaceId: string) => string;
}[] = [
  {
    key: "labels",
    label: "Labels",
    href: (workspaceId) => workspaceHref("settings", workspaceId),
  },
  { key: "overflow", label: "Overflow", href: overflowHref },
];

function TabAnchor({
  label,
  active,
  href,
  onClick,
}: {
  label: string;
  active: boolean;
  /** Set by the router link. */
  href?: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
}) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  return (
    <a
      href={href}
      onClick={onClick}
      aria-current={active ? "page" : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        marginBottom: -1,
        padding: "10px 0",
        borderBottom: `2px solid ${active ? colors.ink : "transparent"}`,
        color: active || hovered ? colors.ink : colors.muted,
        fontFamily: fonts.sans,
        fontSize: 14,
        fontWeight: 500,
        lineHeight: "20px",
        textDecoration: "none",
        whiteSpace: "nowrap",
        ...focus.focusRingVariables,
      }}
    >
      {label}
    </a>
  );
}

export function SettingsTabs({
  workspaceId,
  active,
}: {
  workspaceId: string;
  active: SettingsTab;
}) {
  return (
    <nav
      aria-label="Settings"
      style={{
        display: "flex",
        flexWrap: "wrap",
        columnGap: 24,
        borderBottom: `1px solid ${colors.border}`,
      }}
    >
      {tabs.map((tab) => {
        const href = tab.href(workspaceId);
        return (
          <Link key={tab.key} href={href} asChild>
            <TabAnchor label={tab.label} active={tab.key === active} />
          </Link>
        );
      })}
    </nav>
  );
}
