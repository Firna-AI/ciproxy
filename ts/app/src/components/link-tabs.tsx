// Tabs that are links between sibling views of one page, such as Live and
// History of Jobs. Each tab is a real anchor that the router opens in place,
// so the tab shows in the address and works with the back button. The
// current tab has an ink line under its word, and a tab with live data leads
// with a live dot.

import { useState, type MouseEvent } from "react";
import { StyleSheet, View } from "react-native";
import { Link } from "expo-router";
import { useFocusRing } from "@firna/ui/focusRing";
import { StatusDot } from "@firna/ui/status-dot";

import { colors, fonts, statusColors } from "@ciproxy/tokens";

export type TabLink = {
  key: string;
  label: string;
  href: string;
  /** The tab shows live data, so it leads with a live dot. */
  live?: boolean;
};

type AnchorProps = {
  tab: TabLink;
  current: boolean;
  /** Set by the router link. */
  href?: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
};

function TabAnchor({ tab, current, href, onClick }: AnchorProps) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  return (
    <a
      href={href ?? tab.href}
      onClick={onClick}
      aria-current={current ? "page" : undefined}
      {...focus.focusRingDomProps}
      onFocus={focus.onFocus}
      onBlur={focus.onBlur}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 8,
        marginBottom: -1,
        padding: "10px 0",
        borderBottom: `2px solid ${current ? colors.ink : "transparent"}`,
        color: current || hovered ? colors.ink : colors.muted,
        fontFamily: fonts.sans,
        fontSize: 14,
        fontWeight: 500,
        lineHeight: "20px",
        textDecoration: "none",
        whiteSpace: "nowrap",
        ...focus.focusRingVariables,
      }}
    >
      {tab.live ? <StatusDot color={statusColors.run} size="sm" /> : null}
      {tab.label}
    </a>
  );
}

export function LinkTabs({
  tabs,
  current,
  label,
}: {
  tabs: readonly TabLink[];
  /** The key of the tab that shows now. */
  current: string;
  /** The accessible name of the tab row, such as "Jobs views". */
  label: string;
}) {
  return (
    <View style={styles.tabs} role="navigation" aria-label={label}>
      {tabs.map((tab) => (
        <Link key={tab.key} href={tab.href} asChild>
          <TabAnchor tab={tab} current={tab.key === current} />
        </Link>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  tabs: {
    flexDirection: "row",
    gap: 24,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
});
