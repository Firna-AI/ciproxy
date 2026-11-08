// A notice: a box with a full hairline border, a leading icon, a title, a
// body, and optional actions. A warning or a failure tints the box; the edge
// stays the same all round, with no accent edge.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii, statusColors } from "@ciproxy/tokens";

import type { NoticeTone } from "../view/page-model.ts";
import { AlertIcon, InfoIcon, WarningIcon } from "./icons.tsx";
import { text } from "./text.ts";

const tones = {
  info: { icon: InfoIcon, color: statusColors.run, box: null },
  warning: {
    icon: WarningIcon,
    color: colors.amberDeep,
    box: { backgroundColor: colors.amberSoft, borderColor: colors.amberSoft },
  },
  danger: {
    icon: AlertIcon,
    color: colors.roseDeep,
    box: { backgroundColor: colors.roseSoft, borderColor: colors.roseSoft },
  },
} as const;

export function Notice({
  tone,
  title,
  children,
  actions,
}: {
  tone: NoticeTone;
  title: string;
  children?: ReactNode;
  actions?: ReactNode;
}) {
  const { icon: Icon, color, box } = tones[tone];
  return (
    <View
      style={[styles.notice, box]}
      role={tone === "danger" ? "alert" : "status"}
    >
      <View style={styles.icon}>
        <Icon size={16} color={color} />
      </View>
      <View style={styles.text}>
        <Text style={[text.body, text.strong]}>{title}</Text>
        {children ? <Text style={text.small}>{children}</Text> : null}
      </View>
      {actions ? <View style={styles.actions}>{actions}</View> : null}
    </View>
  );
}

const styles = StyleSheet.create({
  notice: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "flex-start",
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
  },
  icon: { paddingTop: 2 },
  text: { flexGrow: 1, flexShrink: 1, flexBasis: 200, gap: 2, minWidth: 0 },
  actions: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
  },
});
