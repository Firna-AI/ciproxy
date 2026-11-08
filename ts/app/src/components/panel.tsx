// A panel: a white box with a hairline border on every side and small
// corners, with an optional title row. A flush panel lets a table run to its
// edges, and its title row then gets a divider below it. A panel with a title
// is a region that its title names, so assistive technology can move between
// the panels of a page.

import { useId, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { text } from "./text.ts";
import { useViewport } from "./viewport.ts";

export function Panel({
  title,
  action,
  flush = false,
  children,
}: {
  title?: string;
  /** A control at the end of the title row. */
  action?: ReactNode;
  /** The content runs to the panel edges, as a table does. */
  flush?: boolean;
  children: ReactNode;
}) {
  const mobile = useViewport() === "mobile";
  const titleId = useId();
  const head = title !== undefined || action !== undefined;
  const named = title !== undefined;
  return (
    <View
      role={named ? "region" : undefined}
      aria-labelledby={named ? titleId : undefined}
      style={[
        styles.panel,
        mobile && styles.panelMobile,
        flush && styles.flush,
      ]}
    >
      {head ? (
        <View
          style={[
            styles.head,
            flush && styles.flushHead,
            flush && mobile && styles.flushHeadMobile,
          ]}
        >
          {title === undefined ? (
            <View />
          ) : (
            <Text
              id={titleId}
              role="heading"
              aria-level={2}
              style={[text.body, styles.title]}
            >
              {title}
            </Text>
          )}
          {action}
        </View>
      ) : null}
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    padding: 16,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  panelMobile: { padding: 14 },
  flush: { padding: 0, overflow: "hidden" },
  head: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    marginBottom: 12,
  },
  flushHead: {
    minHeight: 48,
    // A wrapping row packs its lines at the top, so center them in the row.
    alignContent: "center",
    marginBottom: 0,
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  flushHeadMobile: { paddingHorizontal: 14 },
  title: { fontWeight: "600" },
});
