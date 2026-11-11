// The tabs of Settings: real links that the router opens in place, with the
// count of open requests on Members. The current tab has an ink underline
// over the hairline of the row. On a phone, the row scrolls sideways when
// the tabs do not fit.

import { useState, type MouseEvent } from "react";
import { Link } from "expo-router";
import { ScrollView, StyleSheet } from "react-native";
import { Badge } from "@firna/ui/badge";
import { useFocusRing } from "@firna/ui/focusRing";

import { colors, fonts } from "@ciproxy/tokens";

import {
  tabItems,
  type SettingsTab,
  type TabItem,
} from "../../view/settings-tabs.ts";

function TabAnchor({
  item,
  href,
  onClick,
}: {
  item: TabItem;
  /** Set by the router link. */
  href?: string;
  onClick?: (event: MouseEvent<HTMLAnchorElement>) => void;
}) {
  const focus = useFocusRing();
  const [hovered, setHovered] = useState(false);
  return (
    <a
      href={href ?? item.href}
      onClick={onClick}
      aria-current={item.active ? "page" : undefined}
      aria-label={item.count === null ? undefined : item.accessibilityLabel}
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
        borderBottom: `2px solid ${item.active ? colors.ink : "transparent"}`,
        color: item.active || hovered ? colors.ink : colors.muted,
        fontFamily: fonts.sans,
        fontSize: 14,
        fontWeight: 500,
        lineHeight: "20px",
        textDecoration: "none",
        whiteSpace: "nowrap",
        ...focus.focusRingVariables,
      }}
    >
      <span>{item.label}</span>
      {item.count === null ? null : (
        <Badge tone="neutral" variant="solid" size="sm">
          {String(item.count)}
        </Badge>
      )}
    </a>
  );
}

export function SettingsTabs({
  workspaceId,
  active,
  openRequests,
}: {
  workspaceId: string;
  active: SettingsTab;
  openRequests: number | null;
}) {
  const items = tabItems(workspaceId, active, openRequests);
  return (
    <ScrollView
      horizontal
      showsHorizontalScrollIndicator={false}
      contentContainerStyle={styles.row}
    >
      <nav
        aria-label="Settings tabs"
        style={{
          display: "flex",
          flexGrow: 1,
          gap: 24,
          borderBottom: `1px solid ${colors.border}`,
        }}
      >
        {items.map((item) => (
          // Each tab stays on /settings and changes only the query, so it
          // pushes a new history entry, and Back returns to the last tab.
          <Link key={item.tab} href={item.href} asChild push>
            <TabAnchor item={item} />
          </Link>
        ))}
      </nav>
    </ScrollView>
  );
}

const styles = StyleSheet.create({ row: { flexGrow: 1 } });
