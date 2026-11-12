// The header of a page under another page, such as one job under Jobs: a
// breadcrumb of links back, the title with its status, a subtitle, and the
// actions at the end. On a phone the actions wrap below the title.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { ChevronIcon } from "./icons.tsx";
import { text } from "./text.ts";
import { TextLink } from "./text-link.tsx";

export type Crumb = { label: string; href: string };

export function DetailHeader({
  crumbs,
  title,
  status,
  subtitle,
  actions,
}: {
  crumbs: readonly Crumb[];
  title: string;
  /** A status beside the title, such as a dot and its word. */
  status?: ReactNode;
  subtitle?: string;
  actions?: ReactNode;
}) {
  return (
    <View style={styles.header}>
      <View style={styles.text}>
        <View style={styles.crumbs} role="navigation" aria-label="Breadcrumb">
          {crumbs.map((crumb) => (
            <View key={crumb.href} style={styles.crumb}>
              <TextLink
                label={crumb.label}
                href={crumb.href}
                tone="muted"
                size="small"
                strong
              />
              <ChevronIcon size={14} color={colors.muted} />
            </View>
          ))}
        </View>
        <View style={styles.titleRow}>
          <Text role="heading" aria-level={1} style={text.pageTitle}>
            {title}
          </Text>
          {status}
        </View>
        {subtitle ? (
          <Text style={[text.small, styles.subtitle]}>{subtitle}</Text>
        ) : null}
      </View>
      {actions ? <View style={styles.actions}>{actions}</View> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  header: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  text: { flexGrow: 1, flexShrink: 1, flexBasis: 280, minWidth: 0, gap: 2 },
  crumbs: { flexDirection: "row", alignItems: "center", gap: 4 },
  crumb: { flexDirection: "row", alignItems: "center", gap: 4 },
  titleRow: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    columnGap: 12,
    rowGap: 4,
  },
  subtitle: { color: colors.muted },
  actions: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
  },
});
