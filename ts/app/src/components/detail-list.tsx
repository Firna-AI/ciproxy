// A details list: rows of a muted label and its value at the end, with a
// hairline between rows. A value can be text or a stack of nodes, such as a
// name with a caption below it.

import { Children, type ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { text } from "./text.ts";

/** The rows of the list. A null or false child leaves no row and no line. */
export function DetailList({ children }: { children: ReactNode }) {
  return (
    <View style={styles.list}>
      {Children.toArray(children).map((child, index) => (
        <View key={index} style={index > 0 ? styles.divided : undefined}>
          {child}
        </View>
      ))}
    </View>
  );
}

/** A label and its value: one row of a details list. */
export function DetailRow({
  label,
  children,
}: {
  label: string;
  children: ReactNode;
}) {
  return (
    <View style={styles.row}>
      <Text style={[text.body, styles.label]}>{label}</Text>
      <View style={styles.value}>
        {typeof children === "string" ? (
          <Text style={[text.body, styles.valueText]}>{children}</Text>
        ) : (
          children
        )}
      </View>
    </View>
  );
}

/** A value with a caption below it, both at the end of the row. */
export function DetailStack({
  children,
  caption,
}: {
  children: ReactNode;
  caption: string;
}) {
  return (
    <View style={styles.stack}>
      {typeof children === "string" ? (
        <Text style={[text.body, styles.valueText]}>{children}</Text>
      ) : (
        children
      )}
      <Text style={[text.caption, styles.valueText]}>{caption}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  /** The first row sits flush with the content above it. */
  list: { marginTop: -10 },
  divided: { borderTopWidth: 1, borderTopColor: colors.border },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    alignItems: "flex-start",
    gap: 16,
    paddingVertical: 10,
  },
  label: { flexShrink: 0, color: colors.muted },
  value: { flexShrink: 1, minWidth: 0, alignItems: "flex-end" },
  valueText: { textAlign: "right" },
  stack: { alignItems: "flex-end", gap: 2 },
});
