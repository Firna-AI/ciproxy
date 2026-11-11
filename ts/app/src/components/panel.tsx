// A panel of a page: a white box with a hairline border, a title, an
// optional action on the right of the title, and its content, as the
// panels of the mockups. A panel whose content changes in place can take
// the focus on its title, so focus never falls back to the page.

import type { ReactNode, RefObject } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { useViewport } from "./viewport.ts";

export function Panel({
  title,
  action,
  titleRef,
  children,
}: {
  title: string;
  action?: ReactNode;
  /** Set to move focus to the title, for example after a change. */
  titleRef?: RefObject<View | null>;
  children: ReactNode;
}) {
  const mobile = useViewport() === "mobile";
  return (
    <View style={[styles.panel, mobile && styles.mobile]}>
      <View style={styles.head}>
        <View ref={titleRef} tabIndex={titleRef ? -1 : undefined}>
          <Text role="heading" aria-level={2} style={styles.title}>
            {title}
          </Text>
        </View>
        {action}
      </View>
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    gap: 12,
    padding: 16,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  mobile: { padding: 14 },
  head: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  title: {
    fontFamily: fonts.sans,
    fontSize: 14,
    fontWeight: "600",
    lineHeight: 20,
    color: colors.ink,
  },
});
