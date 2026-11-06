// A block of code in the mono face, with a label above it. A line that
// starts with - or + reads as a diff line. The lines are DOM code elements,
// so a long token wraps at any character on a phone and stays selectable.

import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { text } from "./text.ts";

export function CodeBlock({
  label,
  lines,
  testID,
}: {
  label: string;
  lines: readonly string[];
  /** A handle for tools, such as the mask of a screenshot. */
  testID?: string;
}) {
  return (
    <View style={styles.block}>
      <Text style={[text.caption, styles.label]}>{label}</Text>
      <View style={styles.body} testID={testID}>
        {lines.map((line, index) => (
          <code
            key={index}
            style={{
              ...lineStyle,
              backgroundColor: line.startsWith("+")
                ? colors.primarySoft
                : undefined,
              color: line.startsWith("-") ? colors.muted : colors.ink,
            }}
          >
            {line}
          </code>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  block: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.surface,
    overflow: "hidden",
  },
  label: {
    paddingVertical: 8,
    paddingHorizontal: 12,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.bg,
  },
  body: { paddingVertical: 10, paddingHorizontal: 12, gap: 2 },
});

const lineStyle = {
  display: "block",
  fontFamily: fonts.mono,
  fontSize: 12.5,
  lineHeight: "20px",
  whiteSpace: "pre-wrap",
  wordBreak: "break-all",
} as const;
