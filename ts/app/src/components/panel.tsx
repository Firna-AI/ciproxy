// A panel: a white box with a hairline border on every side and small
// corners, with an optional title row. A flush panel lets a table run to its
// edges, and its title row then gets a divider below it. A panel with a title
// is a region that its title names, so assistive technology can move between
// the panels of a page. A panel whose content changes in place can take the
// focus on its title, so focus never falls back to the page.

import { useId, type ReactNode, type RefObject } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { text } from "./text.ts";
import { useViewport } from "./viewport.ts";

export function Panel({
  title,
  action,
  flush = false,
  titleRef,
  children,
}: {
  title?: string;
  /** A control at the end of the title row. */
  action?: ReactNode;
  /** The content runs to the panel edges, as a table does. */
  flush?: boolean;
  /** Set to move focus to the title, for example after a change. */
  titleRef?: RefObject<View | null>;
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
        !flush && styles.stack,
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
            <View ref={titleRef} tabIndex={titleRef ? -1 : undefined}>
              <Text
                id={titleId}
                role="heading"
                aria-level={2}
                style={[text.body, styles.title]}
              >
                {title}
              </Text>
            </View>
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
  // The head and each child of a padded panel keep one gap between them.
  stack: { gap: 12 },
  panelMobile: { padding: 14 },
  flush: { padding: 0, overflow: "hidden" },
  head: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  flushHead: {
    minHeight: 48,
    // A wrapping row packs its lines at the top, so center them in the row.
    alignContent: "center",
    paddingHorizontal: 16,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  flushHeadMobile: { paddingHorizontal: 14 },
  title: { fontWeight: "600" },
});
